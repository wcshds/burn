use burn::{
    module::{Module, Param},
    tensor::{backend::Backend, Bool, Tensor},
};

#[derive(Module, Debug)]
pub struct Net<B: Backend> {
    buffer: Param<Tensor<B, 1, Bool>>,
}

impl<B: Backend> Net<B> {
    /// Create a new model from the given record.
    pub fn new_with(record: NetRecord<B>) -> Self {
        Self {
            buffer: record.buffer,
        }
    }

    /// Forward pass of the model.
    pub fn forward(&self, _x: Tensor<B, 2>) -> Tensor<B, 1, Bool> {
        self.buffer.val()
    }
}

#[cfg(test)]
mod tests {

    use burn::{
        record::{FullPrecisionSettings, Recorder},
        tensor::Data,
    };
    use burn_import::pytorch::PyTorchFileRecorder;

    use super::*;

    type Backend = burn_ndarray::NdArray<f32>;

    #[test]
    #[ignore = "It appears loading boolean tensors are not supported yet"]
    // Error skipping: Msg("unsupported storage type BoolStorage")
    fn boolean() {
        let device = Default::default();
        let record = PyTorchFileRecorder::<FullPrecisionSettings>::default()
            .load("tests/boolean/boolean.pt".into(), &device)
            .expect("Should decode state successfully");

        let model = Net::<Backend>::new_with(record);

        let input = Tensor::<Backend, 2>::ones([3, 3], &device);

        let output = model.forward(input);

        let expected =
            Tensor::<Backend, 1, Bool>::from_bool(Data::from([true, false, true]), &device);

        assert_eq!(output.to_data(), expected.to_data());
    }
}
