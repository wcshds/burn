#[burn_tensor_testgen::testgen(module_backward)]
mod tests {
    use super::*;
    use burn_tensor::{backend::Backend, module::embedding, Data, Int, Tensor};

    #[test]
    fn test_embedding_backward() {
        let weights = Data::from([[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]]);
        let indices = Data::from([[0, 1], [1, 1]]);
        let x = Data::from([
            [[1.0, 2.0], [4.0, 5.0], [3.0, 4.0]],
            [[4.0, 5.0], [8.0, 5.0], [1.0, 9.0]],
        ]);
        let device = Default::default();
        let weights = Tensor::<TestAutodiffBackend, 2>::from_data(weights, &device).require_grad();
        let indices = Tensor::<TestAutodiffBackend, 2, Int>::from_data(indices, &device);
        let x = Tensor::<TestAutodiffBackend, 3>::from_data(x, &device).require_grad();

        let output = embedding(weights.clone(), indices);
        let output = output.matmul(x);
        let grads = output.backward();

        let grad = weights.grad(&grads).unwrap();
        let expected = Data::<<TestAutodiffBackend as Backend>::FloatElem, 2>::from([
            [3., 9., 7.],
            [21., 35., 27.],
        ]);
        assert_eq!(grad.to_data(), expected);
    }
}
