#[burn_tensor_testgen::testgen(squeeze)]
mod tests {
    use super::*;
    use burn_tensor::{Data, Shape, Tensor};

    /// Test if the function can successfully squeeze the size 1 dimension of a 3D tensor.
    #[test]
    fn should_squeeze() {
        let tensor = Tensor::<TestBackend, 3>::ones(Shape::new([2, 1, 4]), &Default::default());
        let squeezed_tensor: Tensor<TestBackend, 2> = tensor.squeeze(1);
        let expected_shape = Shape::new([2, 4]);
        assert_eq!(squeezed_tensor.shape(), expected_shape);
    }
    /// Test if the function can successfully squeeze the first size 1 dimension of a 4D tensor.
    #[test]
    fn should_squeeze_first() {
        let tensor = Tensor::<TestBackend, 4>::ones(Shape::new([1, 3, 4, 5]), &Default::default());
        let squeezed_tensor: Tensor<TestBackend, 3> = tensor.squeeze(0);
        let expected_shape = Shape::new([3, 4, 5]);
        assert_eq!(squeezed_tensor.shape(), expected_shape);
    }
    /// Test if the function can successfully squeeze the last size 1 dimension of a 4D tensor.
    #[test]
    fn should_squeeze_last() {
        let tensor = Tensor::<TestBackend, 4>::ones(Shape::new([2, 3, 4, 1]), &Default::default());
        let squeezed_tensor: Tensor<TestBackend, 3> = tensor.squeeze(3);
        let expected_shape = Shape::new([2, 3, 4]);
        assert_eq!(squeezed_tensor.shape(), expected_shape);
    }
    /// Test if the function panics when the squeezed dimension is not of size 1.
    #[test]
    #[should_panic]
    fn should_squeeze_panic() {
        let tensor = Tensor::<TestBackend, 4>::ones(Shape::new([2, 3, 4, 5]), &Default::default());
        let squeezed_tensor: Tensor<TestBackend, 3> = tensor.squeeze(2);
    }

    /// Test if the function can successfully unsqueeze the size 1 dimension at the specified position of a 3D tensor.
    #[test]
    fn should_unsqueeze_dim() {
        let tensor = Tensor::<TestBackend, 3>::ones(Shape::new([2, 4, 1]), &Default::default());
        let unsqueezed_tensor: Tensor<TestBackend, 4> = tensor.unsqueeze_dim(1);
        let expected_shape = Shape::new([2, 1, 4, 1]);
        assert_eq!(unsqueezed_tensor.shape(), expected_shape);
    }

    /// Test if the function can successfully unsqueeze the first size 1 dimension of a 4D tensor.
    #[test]
    fn should_unsqueeze_dim_first() {
        let tensor = Tensor::<TestBackend, 4>::ones(Shape::new([2, 3, 4, 5]), &Default::default());
        let unsqueezed_tensor: Tensor<TestBackend, 5> = tensor.unsqueeze_dim(0);
        let expected_shape = Shape::new([1, 2, 3, 4, 5]);
        assert_eq!(unsqueezed_tensor.shape(), expected_shape);
    }

    /// Test if the function can successfully unsqueeze the last size 1 dimension of a 4D tensor.
    #[test]
    fn should_unsqueeze_dim_last() {
        let tensor = Tensor::<TestBackend, 4>::ones(Shape::new([5, 4, 3, 2]), &Default::default());
        let unsqueezed_tensor: Tensor<TestBackend, 5> = tensor.unsqueeze_dim(4);
        let expected_shape = Shape::new([5, 4, 3, 2, 1]);
        assert_eq!(unsqueezed_tensor.shape(), expected_shape);
    }

    /// Test if the function panics when the unsqueezed dimension is out of bounds.
    #[test]
    #[should_panic]
    fn should_unsqueeze_dim_panic() {
        let tensor = Tensor::<TestBackend, 4>::ones(Shape::new([2, 3, 4, 5]), &Default::default());
        let unsqueezed_tensor: Tensor<TestBackend, 5> = tensor.unsqueeze_dim(5);
    }

    #[test]
    fn should_unsqueeze_dims_support_dim_inference() {
        let input_tensor =
            Tensor::<TestBackend, 3>::ones(Shape::new([3, 4, 5]), &Default::default());
        let output_tensor = input_tensor.unsqueeze_dims(&[1, -2]);
        let expected_shape = Shape::new([3, 1, 4, 1, 5]);
        assert_eq!(output_tensor.shape(), expected_shape);
    }

    #[test]
    fn should_unsqueeze_dims_handle_first_last() {
        let input_tensor =
            Tensor::<TestBackend, 3>::ones(Shape::new([3, 4, 5]), &Default::default());
        let output_tensor = input_tensor.unsqueeze_dims(&[0, 4]);
        let expected_shape = Shape::new([1, 3, 4, 5, 1]);
        assert_eq!(output_tensor.shape(), expected_shape);
    }

    #[test]
    fn should_unsqueeze_dims_work_with_single_dim() {
        //bruh, just call unsqueeze_dim
        let input_tensor =
            Tensor::<TestBackend, 3>::ones(Shape::new([3, 4, 5]), &Default::default());
        let output_tensor: Tensor<TestBackend, 4> = input_tensor.unsqueeze_dims(&[1]);
        let expected_shape = Shape::new([3, 1, 4, 5]);
        assert_eq!(output_tensor.shape(), expected_shape);
    }

    #[test]
    #[should_panic]
    fn should_unsqueeze_dims_panic() {
        let input_tensor =
            Tensor::<TestBackend, 3>::ones(Shape::new([3, 4, 5]), &Default::default());
        let output_tensor: Tensor<TestBackend, 5> = input_tensor.unsqueeze_dims(&[0, -6]);
    }
}
