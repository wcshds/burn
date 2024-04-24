#[burn_tensor_testgen::testgen(ad_sigmoid)]
mod tests {
    use super::*;
    use burn_tensor::{activation, Data};

    #[test]
    fn should_diff_sigmoid() {
        let data = Data::<f32, 1>::from([0.8762]);

        let device = Default::default();
        let tensor_1 = TestAutodiffTensor::from_data(data, &device).require_grad();
        let tensor_2 = activation::sigmoid(tensor_1.clone());
        let grads = tensor_2.backward();

        let grad = tensor_1.grad(&grads).unwrap();

        grad.to_data().assert_approx_eq(&Data::from([0.207549]), 4);
    }

    #[test]
    fn small_neg_val_should_not_cause_grad_overflow() {
        let data = Data::<f32, 1>::from([-90.0]);

        let device = Default::default();
        let tensor_1 = TestAutodiffTensor::from_data(data, &device).require_grad();
        let tensor_2 = activation::sigmoid(tensor_1.clone());
        let grads = tensor_2.backward();

        let grad = tensor_1.grad(&grads).unwrap();

        grad.to_data().assert_approx_eq(&Data::from([0.0]), 4);
    }
}
