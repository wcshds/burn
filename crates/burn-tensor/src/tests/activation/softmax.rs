#[burn_tensor_testgen::testgen(softmax)]
mod tests {
    use super::*;
    use burn_tensor::{activation, Data, Tensor};

    #[test]
    fn test_softmax_d2() {
        let tensor = TestTensor::from([[1.0, 7.0], [13.0, -3.0]]);

        let data_actual = activation::softmax(tensor, 1).into_data();

        let data_expected = Data::from([[2.47e-03, 9.975e-01], [1.0, 1.1254e-07]]);
        data_actual.assert_approx_eq(&data_expected, 4);
    }
}
