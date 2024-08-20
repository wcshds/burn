use super::base::ReduceDimNaive;
use crate::kernel::reduce::SumDim;
use cubecl::cube;
use cubecl::prelude::{Cast, Numeric, Tensor, UInt, ABSOLUTE_POS};

#[cube]
impl<EI: Numeric> ReduceDimNaive<EI> for SumDim {
    type Accumulator = EI;

    fn initialize_naive() -> EI {
        EI::from_int(0)
    }

    fn inner_loop_naive(accumulator: &mut EI, current_value: EI, _i: UInt) {
        *accumulator += current_value;
    }

    fn assign_naive<EO: Numeric>(
        output: &mut Tensor<EO>,
        accumulator: EI,
        _shape_reduce_dim: UInt,
    ) {
        output[ABSOLUTE_POS] = EO::cast_from(accumulator);
    }
}
