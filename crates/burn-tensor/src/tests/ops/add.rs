#[burn_tensor_testgen::testgen(add)]
mod tests {
    use super::*;
    use burn_tensor::backend::Backend;
    use burn_tensor::{Data, Tensor};

    #[test]
    fn test_add_d2() {
        let tensor_1 = TestTensor::from([[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]]);
        let tensor_2 = TestTensor::from([[6.0, 7.0, 8.0], [9.0, 10.0, 11.0]]);

        let data_actual = (tensor_1 + tensor_2).into_data();

        let data_expected = Data::from([[6.0, 8.0, 10.0], [12.0, 14.0, 16.0]]);
        assert_eq!(data_expected, data_actual);
    }

    #[test]
    fn test_add_broadcast() {
        let tensor_1 = TestTensor::from([[0.0, 1.0, 2.0]]);
        let tensor_2 = TestTensor::from([[3.0, 4.0, 5.0], [6.0, 7.0, 8.0]]);

        let data_actual = (tensor_1 + tensor_2).into_data();

        let data_expected = Data::from([[3.0, 5.0, 7.0], [6.0, 8.0, 10.0]]);
        assert_eq!(data_expected, data_actual);
    }

    #[test]
    fn test_add_different_strides_rhs() {
        // We need to execute an operation after `from data` to trigger inplace in some backends.
        // Which is the operation that might be problematic in this case.
        let tensor_1 = TestTensor::from([[0.0, 1.0], [2.0, 3.0]]) * 1;
        let tensor_2 = TestTensor::from([[4.0, 5.0], [6.0, 7.0]]) * 1;

        let data_actual = (tensor_1 + tensor_2.transpose()).into_data();

        let data_expected = Data::from([[4.0, 7.0], [7.0, 10.0]]);
        assert_eq!(data_expected, data_actual);
    }

    #[test]
    fn test_add_different_strides_lhs() {
        // We need to execute an operation after `from data` to trigger inplace in some backends.
        // Which is the operation that might be problematic in this case.
        let tensor_1 = TestTensor::from([[0.0, 1.0], [2.0, 3.0]]) * 1;
        let tensor_2 = TestTensor::from([[4.0, 5.0], [6.0, 7.0]]) * 1;

        let data_actual = (tensor_1.transpose() + tensor_2).into_data();

        let data_expected = Data::from([[4.0, 7.0], [7.0, 10.0]]);
        assert_eq!(data_expected, data_actual);
    }

    #[test]
    fn test_add_different_strides_broadcast() {
        // We need to execute an operation after `from data` to trigger inplace in some backends.
        // Which is the operation that might be problematic in this case.
        let tensor_1 = TestTensor::from([[0.0, 1.0], [2.0, 3.0]]) * 1;
        let tensor_2 = TestTensor::from([[4.0, 5.0]]) * 1;

        let data_actual = (tensor_1.transpose() + tensor_2).into_data();

        let data_expected = Data::from([[4.0, 7.0], [5.0, 8.0]]);
        assert_eq!(data_expected, data_actual);
    }

    #[test]
    fn should_support_add_scalar_ops() {
        let scalar = 2.0;
        let tensor = TestTensor::from([[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]]);

        let output = tensor + scalar;

        let data_actual = output.into_data();
        let data_expected = Data::from([[2.0, 3.0, 4.0], [5.0, 6.0, 7.0]]);
        assert_eq!(data_expected, data_actual);
    }

    #[test]
    fn test_add_d2_int() {
        let tensor_1 = TestTensorInt::from([[0, 1, 2], [3, 4, 5]]);
        let tensor_2 = TestTensorInt::from([[6, 7, 8], [9, 10, 11]]);

        let data_actual = (tensor_1 + tensor_2).into_data();

        let data_expected = Data::from([[6, 8, 10], [12, 14, 16]]);
        assert_eq!(data_expected, data_actual);
    }

    #[test]
    fn test_add_broadcast_int() {
        let tensor_1 = TestTensorInt::from([[0, 1, 2]]);
        let tensor_2 = TestTensorInt::from([[3, 4, 5], [6, 7, 8]]);

        let data_actual = (tensor_1 + tensor_2).into_data();

        let data_expected = Data::from([[3, 5, 7], [6, 8, 10]]);
        assert_eq!(data_expected, data_actual);
    }

    #[test]
    fn should_support_add_scalar_ops_int() {
        let scalar = 2;
        let tensor = TestTensorInt::from([[0, 1, 2], [3, 4, 5]]);

        let output = tensor + scalar;

        let data_actual = output.into_data();
        let data_expected = Data::from([[2, 3, 4], [5, 6, 7]]);
        assert_eq!(data_expected, data_actual);
    }
}
