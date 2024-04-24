#[burn_tensor_testgen::testgen(ad_log)]
mod tests {
    use super::*;
    use burn_tensor::Data;

    #[test]
    fn should_diff_log() {
        let data_1 = Data::<f32, 2>::from([[0.0, 1.0], [3.0, 4.0]]);
        let data_2 = Data::<f32, 2>::from([[6.0, 7.0], [9.0, 10.0]]);

        let device = Default::default();
        let tensor_1 = TestAutodiffTensor::from_data(data_1, &device).require_grad();
        let tensor_2 = TestAutodiffTensor::from_data(data_2, &device).require_grad();

        let tensor_3 = tensor_1.clone().matmul(tensor_2.clone().log());
        let tensor_4 = tensor_3.matmul(tensor_2.clone());
        let grads = tensor_4.backward();

        let grad_1 = tensor_1.grad(&grads).unwrap();
        let grad_2 = tensor_2.grad(&grads).unwrap();

        grad_1
            .to_data()
            .assert_approx_eq(&Data::from([[60.2652, 72.3130], [60.2652, 72.3130]]), 3);
        grad_2
            .to_data()
            .assert_approx_eq(&Data::from([[22.8614, 24.5043], [24.5729, 26.8507]]), 3);
    }
}
