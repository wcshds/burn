use burn_tensor::Shape;

use crate::{
    gpu::{gpu, Elem, Scope, Variable},
    kernel::prng::{cast_uint_to_float, lcg_step, taus_step_0, taus_step_1, taus_step_2},
    tensor::JitTensor,
    JitElement, Runtime,
};

use super::{random, Prng};

pub(crate) struct Bernoulli<E> {
    probability: E,
}

impl<E: JitElement> Prng<E> for Bernoulli<E> {
    fn args(self) -> Vec<E> {
        vec![self.probability]
    }

    fn inner_loop(
        scope: &mut Scope,
        args: Vec<Variable>,
        write_index_base: Variable,
        n_invocations: Variable,
        n_values_per_thread: usize,
        state_0: Variable,
        state_1: Variable,
        state_2: Variable,
        state_3: Variable,
        output: Variable,
    ) {
        let prob = args[0];
        gpu!(
            scope,
            range(0u32, n_values_per_thread).for_each(|i, scope| {
                taus_step_0(scope, state_0);
                taus_step_1(scope, state_1);
                taus_step_2(scope, state_2);
                lcg_step(scope, state_3);

                let int_random = scope.create_local(Elem::UInt);
                gpu!(scope, int_random = state_0 ^ state_1);
                gpu!(scope, int_random = int_random ^ state_2);
                gpu!(scope, int_random = int_random ^ state_3);

                let float_random = scope.create_local(Elem::Float);
                cast_uint_to_float(scope, int_random, float_random);

                let bernoulli = scope.create_local(Elem::Bool);
                gpu!(scope, bernoulli = float_random < prob);

                let write_index = scope.create_local(Elem::UInt);
                gpu!(scope, write_index = i * n_invocations);
                gpu!(scope, write_index += write_index_base);
                gpu!(scope, output[write_index] = bernoulli);
            })
        );
    }

    fn args_length() -> usize {
        1
    }
}

/// Pseudo-random generator with bernoulli distribution
pub fn random_bernoulli<R: Runtime, E: JitElement, const D: usize>(
    shape: Shape<D>,
    device: &R::Device,
    probability: E,
) -> JitTensor<R, E, D> {
    random(shape, device, Bernoulli { probability })
}
