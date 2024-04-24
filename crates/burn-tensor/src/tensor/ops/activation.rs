use crate::tensor::ops::tensor::FloatTensorOps;
use crate::{backend::Backend, ElementConversion};
use core::f64::consts::SQRT_2;

use super::{FloatTensor, FullPrecisionBackend};

/// Activation function operations.
///
/// This trait let backend implementations override activation functions for better performance.
pub trait ActivationOps<B: Backend> {
    /// Applies the LeakyReLU activation function.
    ///
    /// # Arguments
    ///
    /// * `tensor` - The tensor.
    /// * `negative_slope` - The negative_slope value that values smaller than 0 are multiplied with.
    ///
    /// # Returns
    ///
    /// The output tensor.
    fn leaky_relu<const D: usize>(
        tensor: FloatTensor<B, D>,
        negative_slope: super::FloatElem<B>,
    ) -> FloatTensor<B, D> {
        let mask = B::float_lower_elem(tensor.clone(), 0.elem());
        let scaled_tensor = B::float_mul_scalar(tensor.clone(), negative_slope.elem());

        // Update the tensor where the values are `< 0` by `tensor * negative_slope`.
        B::float_mask_where(tensor, mask, scaled_tensor)
    }

    /// Applies the ReLU activation function.
    ///
    /// # Arguments
    ///
    /// * `tensor` - The tensor.
    ///
    /// # Returns
    ///
    /// The output tensor.
    fn relu<const D: usize>(tensor: FloatTensor<B, D>) -> FloatTensor<B, D> {
        let mask = B::float_lower_equal_elem(tensor.clone(), 0.elem());

        B::float_mask_fill(tensor, mask, 0.elem())
    }

    /// Applies the ReLU activation function backward.
    ///
    /// # Arguments
    ///
    /// * `output` - The output tensor.
    ///
    /// # Returns
    ///
    /// The gradient.
    fn relu_backward<const D: usize>(
        output: FloatTensor<B, D>,
        grad: FloatTensor<B, D>,
    ) -> FloatTensor<B, D> {
        let mask = B::float_lower_equal_elem(output, 0.elem());

        B::float_mask_fill(grad, mask, 0.elem())
    }

    /// Applies the Gelu activation function.
    ///
    /// # Arguments
    ///
    /// * `tensor` - The tensor.
    ///
    /// # Returns
    ///
    /// The output tensor.
    fn gelu<const D: usize>(tensor: FloatTensor<B, D>) -> FloatTensor<B, D> {
        let x = B::float_div_scalar(tensor.clone(), SQRT_2.elem());
        let x = B::float_erf(x);
        let x = B::float_add_scalar(x, 1i32.elem());
        let x = B::float_mul(tensor, x);

        B::float_div_scalar(x, 2i32.elem())
    }
    /// Applies the PReLu activation function.
    /// # Arguments
    /// * `tensor` - The input tensor
    /// * `alpha` - The weight tensor
    fn prelu<const D: usize>(
        tensor: FloatTensor<B, D>,
        alpha: FloatTensor<B, D>,
    ) -> FloatTensor<B, D> {
        let mask = B::float_lower_elem(tensor.clone(), 0.elem());
        let scaled_tensor = B::float_mul(tensor.clone(), alpha);
        B::float_mask_where(tensor, mask, scaled_tensor)
    }

    /// Applies the Gelu activation function backward.
    ///
    /// # Arguments
    ///
    /// * `x` - The tensor.
    /// * `grad` - The gradient.
    ///
    /// # Returns
    ///
    /// The output tensor.
    fn gelu_backward<const D: usize>(
        x: FloatTensor<B, D>,
        grad: FloatTensor<B, D>,
    ) -> FloatTensor<B, D> {
        // Derivative of the approximate gelu implementation based on tanh.

        let constant_1 = 0.0356774;
        let constant_2 = 0.797885;
        let constant_3 = 0.0535161;
        let constant_4 = 0.398942;

        let x3 = B::float_powf_scalar(x.clone(), 3.0);

        let c1 = B::float_mul_scalar(x3.clone(), constant_1.elem());
        let c2 = B::float_mul_scalar(x.clone(), constant_2.elem());
        let c3 = B::float_mul_scalar(x3, constant_3.elem());
        let c4 = B::float_mul_scalar(x, constant_4.elem());

        let inner1 = B::float_add(c1, c2);
        let inner2 = B::float_add(c3, c4);

        let tanh = B::float_tanh(inner1);

        let sech = B::float_powf_scalar(tanh.clone(), 2.0);
        let sech = B::float_neg(sech);
        let sech = B::float_add_scalar(sech, 1.elem());

        let y1 = B::float_mul_scalar(tanh, 0.5.elem());
        let y2 = B::float_mul(inner2, sech);
        let y2 = B::float_add_scalar(y2, 0.5.elem());
        let y = B::float_add(y1, y2);

        B::float_mul(y, grad)
    }

    /// Applies the Sigmoid activation function.
    ///
    /// # Arguments
    ///
    /// * `tensor` - The tensor.
    ///
    /// # Returns
    ///
    /// The output tensor.
    fn sigmoid<const D: usize>(tensor: FloatTensor<B, D>) -> FloatTensor<B, D> {
        let tensor_full = B::float_into_full_precision(tensor);
        let tensor_tmp =
            FullPrecisionBackend::<B>::float_exp(FullPrecisionBackend::<B>::float_neg(
                FullPrecisionBackend::<B>::float_log(FullPrecisionBackend::<B>::float_add_scalar(
                    FullPrecisionBackend::<B>::float_exp(FullPrecisionBackend::<B>::float_neg(
                        tensor_full,
                    )),
                    1.0.elem(),
                )),
            ));

        B::float_from_full_precision(tensor_tmp)
    }

    /// Applies the Sigmoid activation function backward.
    ///
    /// # Arguments
    ///
    /// * `output` - The output tensor of the sigmoid function.
    /// * `grad` - The gradient.
    ///
    /// # Returns
    ///
    /// The output tensor.
    fn sigmoid_backward<const D: usize>(
        output: FloatTensor<B, D>,
        grad: FloatTensor<B, D>,
    ) -> FloatTensor<B, D> {
        let value = B::float_mul(
            output.clone(),
            B::float_add_scalar(B::float_neg(output), 1.0.elem()),
        );
        B::float_mul(value, grad)
    }

    /// Applies the LogSigmoid activation function.
    ///
    /// # Arguments
    ///
    /// * `tensor` - The tensor.
    ///
    /// # Returns
    ///
    /// The output tensor.
    fn log_sigmoid<const D: usize>(tensor: FloatTensor<B, D>) -> FloatTensor<B, D> {
        // To avoid overflow, we use the log-sum-exp trick.
        //
        // ```ignore
        // log(sigmoid(x)) = log(1/(1 + exp(-x)))
        //                 = log(1) - log(1 + exp(-x))
        //                 = -log(1 + exp(-x))
        //                 = -log(exp(0) + exp(-x))
        // ```
        // The `exp(t)` of even a moderate-magnitude positive number can be astronomically huge, so we
        // subtract the `max(t, 0)` of each value (where `t = -x` in this case). This results in the
        // following equivalence:
        // ```ignore
        // log(sigmoid(x)) = -(max(-x, 0) + log(exp(-max(-x, 0)) + exp(-x - max(-x, 0))))
        // ```
        //
        // This extends the range of values for which we obtain accurate results.

        // max(-x, 0)
        let tensor_neg = B::float_neg(tensor);
        let mask = B::float_lower_elem(tensor_neg.clone(), 0.elem());
        let max_elem = B::float_mask_fill(tensor_neg.clone(), mask, 0.elem());
        let max_elem_neg = B::float_neg(max_elem.clone());

        // z = exp(-max(-x, 0)) + exp(-x - max(-x, 0))
        let z = B::float_add(
            B::float_exp(max_elem_neg.clone()),
            B::float_exp(B::float_sub(tensor_neg, max_elem.clone())),
        );

        // -max(-x, 0) - log(-z)
        B::float_sub(max_elem_neg, B::float_log(z))
    }

    /// Applies the LogSigmoid activation function backward.
    ///
    /// # Arguments
    ///
    /// * `x` - The input tensor.
    /// * `grad` - The gradient.
    ///
    /// # Returns
    ///
    /// The output gradient.
    fn log_sigmoid_backward<const D: usize>(
        x: FloatTensor<B, D>,
        grad: FloatTensor<B, D>,
    ) -> FloatTensor<B, D> {
        // Derivative of -max(-x, 0) - log(exp(-max(-x, 0)) - exp(-x - max(-x, 0)))) is
        // -max_derive - (-max_derive * exp(-max(-x, 0)) + (-1 - max_derive) * exp(-x - max(-x, 0))) / z
        // where z = exp(-max(-x, 0)) + exp(-x - max(-x, 0))
        //
        // This simplifies to:
        // -max_derive - (z-1)/z if x is >= 0
        // -max_derive + (z-1)/z if x is < 0

        let shape = B::float_shape(&x);
        let device = B::float_device(&x);

        // max(-x, 0)
        let x_neg = B::float_neg(x);
        let mask = B::float_lower_elem(x_neg.clone(), 0.elem()); // -x < 0 or x >= 0
        let max_elem = B::float_mask_fill(x_neg.clone(), mask.clone(), 0.elem());

        // z = exp(-max(-x, 0)) + exp(-x - max(-x, 0))
        let z = B::float_add(
            B::float_exp(B::float_neg(max_elem.clone())),
            B::float_exp(B::float_sub(x_neg, max_elem)),
        );

        // Derivative of max(-x, 0) is 1 if x < 0 or 0 if x >= 0
        let ones = B::float_ones(shape, &device);
        let max_derive = B::float_mask_fill(ones.clone(), mask.clone(), 0.elem());
        let sign = B::float_mask_fill(ones.clone(), mask, (-1).elem());

        // grad * (max_derive - sign * (1 - (1 / z)))
        B::float_mul(
            grad,
            B::float_sub(
                max_derive,
                B::float_mul(sign, B::float_sub(ones, B::float_recip(z))),
            ),
        )
    }
}
