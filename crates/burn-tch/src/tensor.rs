use crate::{element::TchElement, LibTorch, LibTorchDevice};
use burn_tensor::{ops::FloatTensorOps, Data, Shape};
use libc::c_void;
use std::{marker::PhantomData, sync::Arc};

/// A reference to a tensor storage.
///
/// We manually implement `Sync` and `Send` unsafely, so even if we could use `Rc`, it isn't safe.
#[allow(clippy::arc_with_non_send_sync)]
pub type StorageRef = Arc<*mut c_void>;

/// A reference to a tensor storage.
#[derive(PartialEq, Debug, Clone)]
pub enum Storage {
    /// When a tensor is a partial view of another tensor.
    View {
        /// Storage reference for the whole buffer.
        buffer_ref: StorageRef,
        /// Storage reference for the partial buffer.
        view_ref: StorageRef,
    },
    /// When a tensor use all of its buffer.
    Owned {
        /// Storage reference for the whole buffer.
        buffer_ref: StorageRef,
    },
}

impl Storage {
    /// Check if the storage can be used inplace.
    pub fn can_mut(&self) -> bool {
        match self {
            Storage::View {
                buffer_ref: start_ref,
                view_ref,
            } => Arc::strong_count(start_ref) == 1 && Arc::strong_count(view_ref) == 1,
            Storage::Owned {
                buffer_ref: start_ref,
            } => Arc::strong_count(start_ref) == 1,
        }
    }

    /// Get the whole buffer reference.
    pub fn buffer_ref(&self) -> &StorageRef {
        match self {
            Storage::View {
                buffer_ref: start_ref,
                view_ref: _,
            } => start_ref,
            Storage::Owned {
                buffer_ref: start_ref,
            } => start_ref,
        }
    }
}

/// A tensor that uses the tch backend.
#[derive(Debug, PartialEq)]
pub struct TchTensor<E: tch::kind::Element, const D: usize> {
    /// Handle to the tensor. Call methods on this field.
    pub tensor: tch::Tensor,
    /// The tensor's storage
    pub storage: Storage,
    phantom: PhantomData<E>,
}

impl<E: tch::kind::Element, const D: usize> TchTensor<E, D> {
    /// Create a new tensor.
    ///
    /// Note that if the tensor was created from an operation that may reuse the same tensor
    /// storage as the parent, you should use [from_existing](TchTensor::from_existing)
    /// instead.
    pub fn new(tensor: tch::Tensor) -> Self {
        #[allow(clippy::arc_with_non_send_sync)]
        let storage = Storage::Owned {
            buffer_ref: Arc::new(tensor.data_ptr()),
        };

        Self {
            tensor,
            phantom: PhantomData,
            storage,
        }
    }

    /// Create a tensor that was created from an operation executed on a parent tensor.
    ///
    /// If the child tensor shared the same storage as its parent, it will be cloned, effectively
    /// tracking how much tensors point to the same memory space.
    pub fn from_existing(tensor: tch::Tensor, storage_parent: Storage) -> Self {
        let storage_child = tensor.data_ptr();
        let mut is_a_new_tensor = true;

        match &storage_parent {
            Storage::View {
                buffer_ref: start_ref,
                view_ref,
            } => {
                if storage_child == *start_ref.as_ref() || storage_child == *view_ref.as_ref() {
                    is_a_new_tensor = false;
                }
            }
            Storage::Owned {
                buffer_ref: start_ref,
            } => {
                if storage_child == *start_ref.as_ref() {
                    is_a_new_tensor = false;
                }
            }
        };

        let storage = match is_a_new_tensor {
            true => Storage::Owned {
                #[allow(clippy::arc_with_non_send_sync)]
                buffer_ref: Arc::new(storage_child),
            },
            false => storage_parent.clone(),
        };

        Self {
            tensor,
            storage,
            phantom: PhantomData,
        }
    }

    /// Create a tensor that uses a part of its parent tensor such as slice and narrow.
    pub fn partial(tensor: tch::Tensor, storage_parent: Storage) -> Self {
        let storage = Storage::View {
            buffer_ref: storage_parent.buffer_ref().clone(),
            #[allow(clippy::arc_with_non_send_sync)]
            view_ref: Arc::new(tensor.data_ptr()),
        };
        Self {
            tensor,
            storage,
            phantom: PhantomData,
        }
    }
}

impl<E: TchElement, const D: usize> std::ops::Add for TchTensor<E, D> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        LibTorch::float_add(self, rhs)
    }
}

impl<E: tch::kind::Element, const D: usize> TchTensor<E, D> {
    pub(crate) fn shape(&self) -> Shape<D> {
        Shape::from(self.tensor.size())
    }
}

// This is safe since we don't use autodiff from LibTorch.
// Also, atomic reference counting is used to know if the tensor's data can be reused.
// If there are multiple reference on the same tensor, it becomes read only.
unsafe impl<E: tch::kind::Element, const D: usize> Send for TchTensor<E, D> {}
unsafe impl<E: tch::kind::Element, const D: usize> Sync for TchTensor<E, D> {}

impl<P: tch::kind::Element, const D: usize> TchTensor<P, D> {
    /// Execute an operation on a tensor if the data can be reused.
    pub fn mut_ops<
        F: Fn(&mut tch::Tensor) -> tch::Tensor,
        EOut: tch::kind::Element,
        const D_OUT: usize,
    >(
        &mut self,
        func: F,
    ) -> Option<TchTensor<EOut, D_OUT>> {
        if !self.storage.can_mut() {
            return None;
        }

        let data = self.storage.clone();
        Some(TchTensor::from_existing(func(&mut self.tensor), data))
    }
    /// Execute a unary ops reusing the tensor data if possible.
    pub fn unary_ops<FOwn, FRef, EOut: tch::kind::Element, const D_OUT: usize>(
        self,
        fown: FOwn,
        fref: FRef,
    ) -> TchTensor<EOut, D_OUT>
    where
        FOwn: Fn(tch::Tensor) -> tch::Tensor,
        FRef: Fn(&tch::Tensor) -> tch::Tensor,
    {
        if !self.storage.can_mut() {
            return TchTensor::from_existing(fref(&self.tensor), self.storage);
        }

        TchTensor::from_existing(fown(self.tensor), self.storage)
    }

    /// Execute a binary ops reusing the tensor data if possible.
    pub fn binary_ops_tensor<FLMut, FRMut, FRef, EOut: tch::kind::Element, const D_OUT: usize>(
        mut lhs: Self,
        mut rhs: Self,
        flmut: FLMut,
        frmut: FRMut,
        fref: FRef,
    ) -> TchTensor<EOut, D_OUT>
    where
        FLMut: Fn(&mut tch::Tensor, &tch::Tensor) -> tch::Tensor,
        FRMut: Fn(&tch::Tensor, &mut tch::Tensor) -> tch::Tensor,
        FRef: Fn(&tch::Tensor, &tch::Tensor) -> tch::Tensor,
    {
        let lhs_shape = lhs.shape();
        let rhs_shape = rhs.shape();
        let mut out_shape = Shape::new([1; D_OUT]);

        for i in 0..D_OUT {
            out_shape.dims[i] = usize::max(lhs_shape.dims[i], rhs_shape.dims[i]);
        }

        let num_elements_out = out_shape.num_elements();

        // Safe to mut lhs tensor.
        if lhs_shape.num_elements() == num_elements_out {
            if let Some(output) = lhs.mut_ops(|lhs| flmut(lhs, &rhs.tensor)) {
                return output;
            }
        }

        // Safe to mut rhs tensor.
        if rhs_shape.num_elements() == num_elements_out {
            if let Some(output) = rhs.mut_ops(|rhs| frmut(&lhs.tensor, rhs)) {
                return output;
            }
        }

        let storage = lhs.storage;
        let tensor = fref(&lhs.tensor, &rhs.tensor);

        TchTensor::from_existing(tensor, storage)
    }
}

impl<P: tch::kind::Element, const D: usize> Clone for TchTensor<P, D> {
    fn clone(&self) -> Self {
        Self {
            tensor: self.tensor.shallow_clone(),
            phantom: PhantomData,
            storage: self.storage.clone(),
        }
    }
}

/// A shape that can be used by LibTorch.
#[derive(Debug)]
pub struct TchShape<const D: usize> {
    /// The shape's dimensions.
    pub dims: [i64; D],
}

impl<const D: usize> From<Shape<D>> for TchShape<D> {
    fn from(shape: Shape<D>) -> Self {
        let mut dims = [0; D];
        for (i, dim) in dims.iter_mut().enumerate().take(D) {
            *dim = shape.dims[i] as i64;
        }
        TchShape { dims }
    }
}

impl<E: tch::kind::Element + Default, const D: usize> TchTensor<E, D> {
    /// Creates a new tensor from a shape and a device.
    ///
    /// # Arguments
    ///
    /// * `data` - The tensor's data.
    /// * `device` - The device on which the tensor will be allocated.
    ///
    /// # Returns
    ///
    /// A new tensor.
    pub fn from_data(data: Data<E, D>, device: tch::Device) -> Self {
        let tensor = tch::Tensor::from_slice(data.value.as_slice()).to(device);
        let shape_tch = TchShape::from(data.shape);
        let tensor = tensor.reshape(shape_tch.dims).to_kind(E::KIND);

        Self::new(tensor)
    }
}

#[cfg(test)]
mod utils {
    use super::*;
    use crate::{backend::LibTorch, element::TchElement};

    impl<P: TchElement, const D: usize> TchTensor<P, D> {
        pub(crate) fn into_data(self) -> Data<P, D>
        where
            P: tch::kind::Element,
        {
            <LibTorch<P> as FloatTensorOps<LibTorch<P>>>::float_into_data(self).read()
        }
    }
}

impl<E: tch::kind::Element + Default + Copy + std::fmt::Debug, const D: usize> TchTensor<E, D> {
    /// Creates an empty tensor from a shape and a device.
    ///
    /// # Arguments
    ///
    /// * `shape` - The shape of the tensor.
    /// * `device` - The device to create the tensor on.
    ///
    /// # Returns
    ///
    /// A new empty tensor.
    pub fn empty(shape: Shape<D>, device: LibTorchDevice) -> Self {
        let shape_tch = TchShape::from(shape);
        let tensor = tch::Tensor::empty(shape_tch.dims, (E::KIND, device.into()));

        Self::new(tensor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn_tensor::{Distribution, Tensor};
    use rand::prelude::StdRng;
    use rand::SeedableRng;

    #[test]
    fn should_support_into_and_from_data_1d() {
        let data_expected = Data::<f32, 1>::random(
            Shape::new([3]),
            Distribution::Default,
            &mut StdRng::from_entropy(),
        );
        let tensor = TchTensor::from_data(data_expected.clone(), tch::Device::Cpu);

        let data_actual = tensor.into_data();

        assert_eq!(data_expected, data_actual);
    }

    #[test]
    fn should_support_into_and_from_data_2d() {
        let data_expected = Data::<f32, 2>::random(
            Shape::new([2, 3]),
            Distribution::Default,
            &mut StdRng::from_entropy(),
        );
        let tensor = TchTensor::from_data(data_expected.clone(), tch::Device::Cpu);

        let data_actual = tensor.into_data();

        assert_eq!(data_expected, data_actual);
    }

    #[test]
    fn should_not_update_inplace_after_reshape() {
        let tensor_1 = Tensor::<LibTorch<f32>, 1>::from_floats([4.0, 4.0], &Default::default());
        let tensor_2 = tensor_1.clone();

        let tensor_3 = tensor_2.reshape([1, 2]).add_scalar(2.0);

        assert_ne!(tensor_3.to_data().value, tensor_1.to_data().value);
    }

    #[test]
    fn should_not_update_inplace_after_slice() {
        let tensor_1 = Tensor::<LibTorch<f32>, 1>::from_floats([4.0, 4.0], &Default::default());
        let tensor_2 = tensor_1.clone();

        let tensor_3 = tensor_2.slice([0..2]).add_scalar(2.0);

        assert_ne!(tensor_3.to_data().value, tensor_1.to_data().value);
    }
}
