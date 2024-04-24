#[burn_tensor_testgen::testgen(gather_scatter)]
mod tests {
    use super::*;
    use burn_tensor::{Data, Tensor};

    #[test]
    fn should_gather_1d_dim0() {
        let device = Default::default();
        let tensor = TestTensor::from_floats([0.0, 1.0, 2.0], &device);
        let indices = TestTensorInt::from_ints([1, 1, 0, 1, 2], &device);

        let output = tensor.gather(0, indices);

        assert_eq!(output.into_data(), Data::from([1.0, 1.0, 0.0, 1.0, 2.0]));
    }

    #[test]
    fn should_gather_1d_dim0_int() {
        let device = Default::default();
        let tensor = TestTensorInt::from_ints([5, 6, 7], &device);
        let indices = TestTensorInt::from_ints([1, 1, 0, 1, 2], &device);

        let output = tensor.gather(0, indices);

        assert_eq!(output.into_data(), Data::from([6, 6, 5, 6, 7]));
    }

    #[test]
    fn should_gather_2d_dim0() {
        let device = Default::default();
        let tensor = TestTensor::from_floats([[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]], &device);
        let indices = TestTensorInt::from_ints([[0, 1, 0], [1, 0, 1]], &device);

        let output = tensor.gather(0, indices);

        assert_eq!(
            output.into_data(),
            Data::from([[0.0, 4.0, 2.0], [3.0, 1.0, 5.0]])
        );
    }

    #[test]
    fn should_gather_2d_dim1() {
        let device = Default::default();
        let tensor = TestTensor::from_floats([[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]], &device);
        let indices = TestTensorInt::from_ints([[2, 1, 0, 0], [2, 0, 1, 2]], &device);

        let output = tensor.gather(1, indices);

        assert_eq!(
            output.into_data(),
            Data::from([[2.0, 1.0, 0.0, 0.0], [5.0, 3.0, 4.0, 5.0]])
        );
    }

    #[test]
    fn should_gather_3d_dim1() {
        let device = Default::default();
        let tensor = TestTensor::from_floats(
            [
                [[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]],
                [[6.0, 7.0, 8.0], [9.0, 10.0, 11.0]],
            ],
            &device,
        );
        let indices =
            TestTensorInt::from_ints([[[1, 0, 0], [0, 1, 0]], [[0, 0, 1], [0, 1, 1]]], &device);

        let output = tensor.gather(1, indices);

        assert_eq!(
            output.into_data(),
            Data::from([
                [[3.0, 1.0, 2.0], [0.0, 4.0, 2.0]],
                [[6.0, 7.0, 11.0], [6.0, 10.0, 11.0]]
            ])
        );
    }

    #[test]
    fn should_gather_2d_only_1dim() {
        let device = Default::default();
        let tensor = TestTensor::from_floats([[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]], &device);
        let indices = TestTensorInt::from_ints([[1, 2]], &device).reshape([2, 1]);

        let output = tensor.gather(1, indices);

        assert_eq!(output.into_data(), Data::from([[1.0], [5.0]]));
    }

    #[test]
    fn should_scatter_1d() {
        let device = Default::default();
        let tensor = TestTensor::from_floats([0.0, 0.0, 0.0], &device);
        let values = TestTensor::from_floats([5.0, 4.0, 3.0], &device);
        let indices = TestTensorInt::from_ints([1, 0, 2], &device);

        let output = tensor.scatter(0, indices, values);

        assert_eq!(output.into_data(), Data::from([4.0, 5.0, 3.0]));
    }

    #[test]
    fn should_scatter_1d_int() {
        let device = Default::default();
        let tensor = TestTensorInt::from_ints([0, 0, 0], &device);
        let values = TestTensorInt::from_ints([5, 4, 3], &device);
        let indices = TestTensorInt::from_ints([1, 0, 2], &device);

        let output = tensor.scatter(0, indices, values);

        assert_eq!(output.into_data(), Data::from([4, 5, 3]));
    }

    #[test]
    fn should_scatter_2d_dim0() {
        let device = Default::default();
        let tensor = TestTensor::from_floats([[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]], &device);
        let values = TestTensor::from_floats([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]], &device);
        let indices = TestTensorInt::from_ints([[1, 0, 1], [1, 1, 0]], &device);

        let output = tensor.scatter(0, indices, values);

        assert_eq!(
            output.into_data(),
            Data::from([[0.0, 2.0, 6.0], [5.0, 5.0, 3.0]])
        );
    }

    #[test]
    fn should_scatter_2d_dim1() {
        let device = Default::default();
        let tensor = TestTensor::from_floats([[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]], &device);
        let values = TestTensor::from_floats([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]], &device);
        let indices = TestTensorInt::from_ints([[1, 0, 2], [1, 2, 0]], &device);

        let output = tensor.scatter(1, indices, values);

        assert_eq!(
            output.into_data(),
            Data::from([[2.0, 1.0, 3.0], [6.0, 4.0, 5.0]])
        );
    }

    #[test]
    fn should_scatter_3d_dim1() {
        let device = Default::default();
        let tensor = TestTensor::from_floats(
            [
                [[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]],
                [[6.0, 7.0, 8.0], [9.0, 10.0, 11.0]],
            ],
            &device,
        );
        let values = TestTensor::from_floats(
            [
                [[12.0, 13.0, 14.0], [15.0, 16.0, 17.0]],
                [[18.0, 19.0, 20.0], [21.0, 22.0, 23.0]],
            ],
            &device,
        );
        let indices =
            TestTensorInt::from_ints([[[1, 0, 0], [0, 1, 0]], [[0, 0, 1], [0, 1, 1]]], &device);

        let output = tensor.scatter(1, indices, values);

        assert_eq!(
            output.into_data(),
            Data::from([
                [[15.0, 14.0, 33.0], [15.0, 20.0, 5.0]],
                [[45.0, 26.0, 8.0], [9.0, 32.0, 54.0]]
            ])
        );
    }

    #[test]
    fn should_scatter_2d_dim1_diff_shape() {
        let device = Default::default();
        let tensor = TestTensor::from_floats([[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]], &device);
        let values = TestTensor::from_floats([[1.0], [4.0]], &device);
        let indices = TestTensorInt::from_ints([[1], [2]], &device);

        let output = tensor.scatter(1, indices, values);

        assert_eq!(
            output.into_data(),
            Data::from([[0.0, 1.0, 0.0], [0.0, 0.0, 4.0]])
        );
    }

    #[test]
    #[should_panic]
    fn scatter_should_panic_on_mismatch_of_shapes() {
        let device = Default::default();
        let tensor = TestTensor::from_floats([0.0, 0.0, 0.0], &device);
        let values = TestTensor::from_floats([5.0, 4.0], &device);
        let indices = TestTensorInt::from_ints([1, 0, 2], &device);

        tensor.scatter(0, indices, values);
    }
}
