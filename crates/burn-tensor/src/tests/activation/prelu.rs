#[burn_tensor_testgen::testgen(prelu)]
mod tests {
    use super::*;
    use burn_tensor::{activation, Data, Tensor};

    #[test]
    fn test_prelu_2_dimension() {
        let data = [
            [-1.1, 0.0, 1.2, 0.25, -5.4],
            [-4.567, 0.56, -1.55, 99.9, 0.0],
        ];
        let tensor = TestTensor::from(data);
        let data_actual =
            activation::prelu(tensor, TestTensor::from([0.5, 0.25, 0.0, -0.8, -0.4])).into_data();
        let data_expected = Data::from([
            [-0.5500, 0.0000, 1.2000, 0.2500, 2.1600],
            [-2.2835, 0.5600, -0.0000, 99.9000, -0.0000],
        ]);
        data_expected.assert_approx_eq(&data_actual, 9);
    }
    #[test]
    fn test_prelu_2_dimension_scalar_weight() {
        let data = [
            [-1.1, 0.0, 1.2, 0.25, -5.4],
            [-4.567, 0.56, -1.55, 99.9, 0.0],
        ];
        let tensor = TestTensor::from(data);
        let data_actual = activation::prelu(tensor, TestTensor::from([-0.8])).into_data();
        let data_expected = Data::from([
            [0.8800, -0.0000, 1.2000, 0.2500, 4.3200],
            [3.6536, 0.5600, 1.2400, 99.9000, -0.0000],
        ]);
        data_expected.assert_approx_eq(&data_actual, 7);
    }

    #[test]
    fn test_prelu_positives() {
        // Check that positives are untouched
        let data = [[
            0.5447, 0.9809, 0.4114, 0.1398, 0.8045, 0.4103, 0.2388, 0.5262, 0.6677, 0.6737,
        ]];
        let tensor = TestTensor::from(data);
        let data_actual = activation::prelu(tensor, TestTensor::from([0.25])).into_data();
        let data_expected = Data::from(data);
        data_expected.assert_approx_eq(&data_actual, 9);
    }
    #[test]
    fn test_prelu_zero_weight() {
        // test that with weight 0 it behaves as relu
        let data = [-1.1, 0.0, 1.2, 0.25, -5.4];
        let tensor = TestTensor::from(data);
        let data_actual = activation::prelu(tensor, TestTensor::from([0.0])).into_data();
        let data_expected = Data::from([0.0, 0.0, 1.2, 0.25, 0.0]);
        data_expected.assert_approx_eq(&data_actual, 9);
    }
    #[test]
    fn test_prelu_some_weight() {
        // test that with some non zero weight it works like leaky relu
        let data = [-1.1, 0.0, 1.2, 0.25, -5.4];
        let tensor = TestTensor::from(data);
        let data_actual = activation::prelu(tensor, TestTensor::from([0.5])).into_data();
        let data_expected = Data::from([-0.550, 0.0, 1.20, 0.250, -2.70]);
        data_expected.assert_approx_eq(&data_actual, 9);
    }
    #[test]
    #[should_panic]
    fn test_prelu_single_dim_multi_weight() {
        // should panic because the data has only 1 channel
        let data = [-1.1, 2.0, 1.2, 0.25, -5.4];
        let tensor = TestTensor::from(data);
        let data_actual =
            activation::prelu(tensor, TestTensor::from([0.5, -0.25, 0.0, 0.5, -1.0])).into_data();
        let data_expected = Data::from([-0.550, 0.0, 1.20, 0.250, -2.70]);
        data_expected.assert_approx_eq(&data_actual, 9);
    }
    #[test]
    #[should_panic]
    fn test_prelu_multi_dim_wrong_weights() {
        let data = [
            [-1.1, 0.0, 1.2, 0.25, -5.4],
            [-4.567, 0.56, -1.55, 99.9, 0.0],
        ];
        let tensor = TestTensor::from(data);
        let data_actual = activation::prelu(tensor, TestTensor::from([-0.8, 0.1])).into_data();
    }
}
