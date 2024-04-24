#[burn_tensor_testgen::testgen(arange_step)]
mod tests {
    use super::*;
    use burn_tensor::backend::Backend;
    use burn_tensor::{Data, Int, Tensor};

    #[test]
    fn test_arange_step() {
        let device = <TestBackend as Backend>::Device::default();

        // Test correct sequence of numbers when the range is 0..9 and the step is 1
        let tensor = Tensor::<TestBackend, 1, Int>::arange_step(0..9, 1, &device);
        assert_eq!(tensor.into_data(), Data::from([0, 1, 2, 3, 4, 5, 6, 7, 8]));

        // Test correct sequence of numbers when the range is 0..3 and the step is 2
        let tensor = Tensor::<TestBackend, 1, Int>::arange_step(0..3, 2, &device);
        assert_eq!(tensor.into_data(), Data::from([0, 2]));

        // Test correct sequence of numbers when the range is 0..2 and the step is 5
        let tensor = Tensor::<TestBackend, 1, Int>::arange_step(0..2, 5, &device);
        assert_eq!(tensor.into_data(), Data::from([0]));

        // Test correct sequence of numbers when the range includes negative numbers
        let tensor = Tensor::<TestBackend, 1, Int>::arange_step(-3..3, 2, &device);
        assert_eq!(tensor.into_data(), Data::from([-3, -1, 1]));

        let tensor = Tensor::<TestBackend, 1, Int>::arange_step(-5..1, 5, &device);
        assert_eq!(tensor.clone().into_data(), Data::from([-5, 0]));
        assert_eq!(tensor.device(), device);
    }

    #[test]
    #[should_panic]
    fn should_panic_when_step_is_zero() {
        let device = <TestBackend as Backend>::Device::default();
        // Test that arange_step panics when the step is 0
        let _tensor = Tensor::<TestBackend, 1, Int>::arange_step(0..3, 0, &device);
    }
}
