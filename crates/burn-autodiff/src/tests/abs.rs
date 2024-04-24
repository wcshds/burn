#[burn_tensor_testgen::testgen(ad_abs)]
mod tests {
    use super::*;
    use burn_tensor::Data;

    #[test]
    fn should_diff_abs() {
        let data_1 = Data::<f32, 2>::from([[0.0, -1.0], [3.0, 4.0]]);
        let data_2 = Data::<f32, 2>::from([[6.0, 7.0], [9.0, -10.0]]);

        let device = Default::default();
        let tensor_1 = TestAutodiffTensor::from_data(data_1, &device).require_grad();
        let tensor_2 = TestAutodiffTensor::from_data(data_2, &device).require_grad();

        let tensor_3 = tensor_1.clone().matmul(tensor_2.clone().abs());
        let tensor_4 = tensor_3.matmul(tensor_2.clone());
        let grads = tensor_4.backward();

        let grad_1 = tensor_1.grad(&grads).unwrap();
        let grad_2 = tensor_2.grad(&grads).unwrap();

        grad_1
            .to_data()
            .assert_approx_eq(&Data::from([[71.0, 107.0], [71.0, 107.0]]), 3);
        grad_2
            .to_data()
            .assert_approx_eq(&Data::from([[84.0, 42.0], [90.0, 54.0]]), 3);
    }
}
