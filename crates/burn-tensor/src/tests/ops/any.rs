#[burn_tensor_testgen::testgen(any)]
mod tests {
    use super::*;
    use burn_tensor::{Data, Tensor};

    #[test]
    fn test_any() {
        // test float tensor
        let tensor = TestTensor::from([[0.0, 0.0, 0.0], [1.0, -1.0, 0.0]]);
        let data_actual = tensor.any().into_data();
        let data_expected = Data::from([true]);
        assert_eq!(data_expected, data_actual);

        let tensor = TestTensor::from([[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]]);
        let data_actual = tensor.any().into_data();
        let data_expected = Data::from([false]);
        assert_eq!(data_expected, data_actual);

        // test int tensor
        let tensor = TestTensorInt::from([[0, 0, 0], [1, -1, 0]]);
        let data_actual = tensor.any().into_data();
        let data_expected = Data::from([true]);
        assert_eq!(data_expected, data_actual);

        let tensor = TestTensorInt::from([[0, 0, 0], [0, 0, 0]]);
        let data_actual = tensor.any().into_data();
        let data_expected = Data::from([false]);
        assert_eq!(data_expected, data_actual);

        // test bool tensor
        let tensor = TestTensorBool::from([[false, false, false], [true, true, false]]);
        let data_actual = tensor.any().into_data();
        let data_expected = Data::from([true]);
        assert_eq!(data_expected, data_actual);

        let tensor = TestTensorBool::from([[false, false, false], [false, false, false]]);
        let data_actual = tensor.any().into_data();
        let data_expected = Data::from([false]);
        assert_eq!(data_expected, data_actual);
    }

    #[test]
    fn test_any_dim() {
        let tensor = TestTensor::from([[0.0, 0.0, 0.0], [1.0, -1.0, 0.0]]);
        let data_actual = tensor.any_dim(1).into_data();
        let data_expected = Data::from([[false], [true]]);
        assert_eq!(data_expected, data_actual);

        // test int tensor
        let tensor = TestTensorInt::from([[0, 0, 0], [1, -1, 0]]);
        let data_actual = tensor.any_dim(1).into_data();
        let data_expected = Data::from([[false], [true]]);
        assert_eq!(data_expected, data_actual);

        // test bool tensor
        let tensor = TestTensorBool::from([[false, false, false], [true, true, false]]);
        let data_actual = tensor.any_dim(1).into_data();
        let data_expected = Data::from([[false], [true]]);
        assert_eq!(data_expected, data_actual);
    }
}
