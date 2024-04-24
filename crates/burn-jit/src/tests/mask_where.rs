#[burn_tensor_testgen::testgen(mask_where)]
mod tests {
    use super::*;
    use burn_jit::kernel::{mask_where, MaskWhereStrategy};
    use burn_tensor::{backend::Backend, Bool, Distribution, Tensor};

    #[test]
    fn mask_where_should_work_with_multiple_invocations() {
        let (tensor, value, mask, tensor_ref, value_ref, mask_ref) = inputs_mask_where();

        let actual = tensor.mask_where(mask, value);
        let expected = tensor_ref.mask_where(mask_ref, value_ref);

        expected
            .into_data()
            .assert_approx_eq(&actual.into_data(), 3);
    }
    #[test]
    fn mask_where_inplace_lhs_should_work_with_multiple_invocations() {
        let (tensor, value, mask, tensor_ref, value_ref, mask_ref) = inputs_mask_where();

        let actual = Tensor::<TestBackend, 3>::from_primitive(mask_where(
            tensor.into_primitive(),
            mask.into_primitive(),
            value.into_primitive(),
            MaskWhereStrategy::InplaceLhs,
        ));
        let expected = tensor_ref.mask_where(mask_ref, value_ref);

        expected
            .into_data()
            .assert_approx_eq(&actual.into_data(), 3);
    }

    #[test]
    fn mask_where_inplace_rhs_should_work_with_multiple_invocation() {
        let (tensor, value, mask, tensor_ref, value_ref, mask_ref) = inputs_mask_where();

        let actual = Tensor::<TestBackend, 3>::from_primitive(mask_where(
            tensor.into_primitive(),
            mask.into_primitive(),
            value.into_primitive(),
            MaskWhereStrategy::InplaceRhs,
        ));
        let expected = tensor_ref.mask_where(mask_ref, value_ref);

        expected
            .into_data()
            .assert_approx_eq(&actual.into_data(), 3);
    }

    #[allow(clippy::type_complexity)]
    fn inputs_mask_where() -> (
        Tensor<TestBackend, 3>,
        Tensor<TestBackend, 3>,
        Tensor<TestBackend, 3, Bool>,
        Tensor<ReferenceBackend, 3>,
        Tensor<ReferenceBackend, 3>,
        Tensor<ReferenceBackend, 3, Bool>,
    ) {
        TestBackend::seed(0);
        let device = Default::default();
        let tensor = Tensor::<TestBackend, 3>::random([2, 6, 256], Distribution::Default, &device);
        let value = Tensor::<TestBackend, 3>::random([2, 6, 256], Distribution::Default, &device);
        let mask =
            Tensor::<TestBackend, 3>::random([2, 6, 256], Distribution::Uniform(0., 1.), &device)
                .lower_equal_elem(0.5);

        let device_ref = Default::default();
        let tensor_ref = Tensor::<ReferenceBackend, 3>::from_data(tensor.to_data(), &device_ref);
        let value_ref = Tensor::<ReferenceBackend, 3>::from_data(value.to_data(), &device_ref);
        let mask_ref = Tensor::<ReferenceBackend, 3, Bool>::from_data(mask.to_data(), &device_ref);
        assert_eq!(mask.to_data(), mask_ref.to_data());

        (tensor, value, mask, tensor_ref, value_ref, mask_ref)
    }
}
