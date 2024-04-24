#[burn_tensor_testgen::testgen(sub)]
mod tests {
    use super::*;
    use burn_tensor::{Data, Int, Tensor};

    #[test]
    fn should_support_sub_ops() {
        let data_1 = Data::from([[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]]);
        let data_2 = Data::from([[6.0, 7.0, 8.0], [9.0, 10.0, 11.0]]);
        let data_expected = Data::from([[-6.0, -6.0, -6.0], [-6.0, -6.0, -6.0]]);
        let device = Default::default();
        let tensor_1 = Tensor::<TestBackend, 2>::from_data(data_1, &device);
        let tensor_2 = Tensor::<TestBackend, 2>::from_data(data_2, &device);

        let data_actual = (tensor_1 - tensor_2).into_data();

        assert_eq!(data_expected, data_actual);
    }

    #[test]
    fn test_sub_broadcast() {
        let data_1 = Data::from([[0.0, 1.0, 2.0]]);
        let data_2 = Data::from([[3.0, 4.0, 5.0], [6.0, 7.0, 8.0]]);
        let device = Default::default();
        let tensor_1 = Tensor::<TestBackend, 2>::from_data(data_1, &device);
        let tensor_2 = Tensor::<TestBackend, 2>::from_data(data_2, &device);

        let data_actual = (tensor_1 - tensor_2).into_data();

        let data_expected = Data::from([[-3.0, -3.0, -3.0], [-6.0, -6.0, -6.0]]);
        assert_eq!(data_expected, data_actual);
    }

    #[test]
    fn should_support_sub_scalar_ops() {
        let data = Data::from([[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]]);
        let scalar = 2.0;
        let tensor = Tensor::<TestBackend, 2>::from_data(data, &Default::default());

        let output = tensor - scalar;

        let data_actual = output.into_data();
        let data_expected = Data::from([[-2.0, -1.0, 0.0], [1.0, 2.0, 3.0]]);
        assert_eq!(data_expected, data_actual);
    }

    #[test]
    fn should_support_sub_ops_int() {
        let data_1 = Data::from([[0, 1, 2], [3, 4, 5]]);
        let data_2 = Data::from([[6, 7, 8], [9, 10, 11]]);
        let data_expected = Data::from([[-6, -6, -6], [-6, -6, -6]]);
        let device = Default::default();
        let tensor_1 = Tensor::<TestBackend, 2, Int>::from_data(data_1, &device);
        let tensor_2 = Tensor::<TestBackend, 2, Int>::from_data(data_2, &device);

        let data_actual = (tensor_1 - tensor_2).into_data();

        assert_eq!(data_expected, data_actual);
    }

    #[test]
    fn test_sub_broadcast_int() {
        let data_1 = Data::from([[0, 1, 2]]);
        let data_2 = Data::from([[3, 4, 5], [6, 7, 8]]);
        let device = Default::default();
        let tensor_1 = Tensor::<TestBackend, 2, Int>::from_data(data_1, &device);
        let tensor_2 = Tensor::<TestBackend, 2, Int>::from_data(data_2, &device);

        let data_actual = (tensor_1 - tensor_2).into_data();

        let data_expected = Data::from([[-3, -3, -3], [-6, -6, -6]]);
        assert_eq!(data_expected, data_actual);
    }

    #[test]
    fn should_support_sub_scalar_ops_int() {
        let data = Data::from([[0, 1, 2], [3, 4, 5]]);
        let scalar = 2;
        let tensor = Tensor::<TestBackend, 2, Int>::from_data(data, &Default::default());

        let output = tensor - scalar;

        let data_actual = output.into_data();
        let data_expected = Data::from([[-2, -1, 0], [1, 2, 3]]);
        assert_eq!(data_expected, data_actual);
    }
}
