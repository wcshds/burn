#![warn(missing_docs)]
#![allow(clippy::single_range_in_vec_init)]

//! Burn Tch Backend

mod backend;
mod bridge;
mod element;
mod ops;
mod tensor;

pub use backend::*;
pub use bridge::*;
pub use element::*;
pub use tensor::*;

#[cfg(test)]
mod tests {
    extern crate alloc;

    type TestBackend = crate::LibTorch<f32>;
    type TestTensor<const D: usize> = burn_tensor::Tensor<TestBackend, D>;
    type TestTensorInt<const D: usize> = burn_tensor::Tensor<TestBackend, D, burn_tensor::Int>;
    type TestTensorBool<const D: usize> = burn_tensor::Tensor<TestBackend, D, burn_tensor::Bool>;

    burn_tensor::testgen_all!();
    burn_autodiff::testgen_all!();
}
