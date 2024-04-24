use crate::{
    codegen::dialect::gpu::{gpu, Elem, Item, Scope, Variable},
    JitElement,
};

use super::ReduceDimAlgorithm;

pub(crate) struct ArgMax;

impl<E: JitElement> ReduceDimAlgorithm<E> for ArgMax {
    type Accumulator = (Variable, Variable);

    fn initialize_naive(
        scope: &mut Scope,
        input_item: Item,
        _output_item: Item,
    ) -> Self::Accumulator {
        let index = scope.create_local(Elem::UInt);
        let max = scope.create_local(input_item);
        let max_initial =
            Variable::ConstantScalar(E::minimum_value().to_f64().unwrap(), input_item.elem());
        gpu!(scope, max = max_initial);

        (max, index)
    }

    fn inner_loop_naive(
        scope: &mut Scope,
        (max, index): Self::Accumulator,
        value: Variable,
        i: Variable,
    ) {
        let condition = scope.create_local(Elem::Bool);
        gpu!(scope, condition = value > max);
        gpu!(scope, if(condition).then(|scope| {
            gpu!(scope, max = value);
            gpu!(scope, index = i);
        }));
    }

    fn assign_naive(
        scope: &mut Scope,
        output: Variable,
        (_max, index): Self::Accumulator,
        _shape_reduce_dim: Variable,
    ) {
        let id = Variable::Id;
        gpu!(scope, output[id] = index);
    }

    fn initialize_shared(
        scope: &mut Scope,
        shared_memory_size: u32,
        write_position: Variable,
        input_item: Item,
    ) -> Self::Accumulator {
        let value_shared_memory = scope.create_shared(input_item, shared_memory_size);
        let index_shared_memory = scope.create_shared(Elem::UInt, shared_memory_size);

        let max = Variable::ConstantScalar(E::minimum_value().to_f64().unwrap(), input_item.elem());
        gpu!(scope, value_shared_memory[write_position] = max);
        (value_shared_memory, index_shared_memory)
    }

    fn write_to_shared(
        scope: &mut Scope,
        shared_memory: Self::Accumulator,
        write_position: Variable,
        (value, index): Self::Accumulator,
    ) {
        let (value_shared_memory, index_shared_memory) = shared_memory;
        let current_value = scope.create_local(value.item());
        gpu!(scope, current_value = value_shared_memory[write_position]);

        let condition = scope.create_local(Elem::Bool);
        gpu!(scope, condition = value > current_value);
        gpu!(scope, if(condition).then(|scope| {
            gpu!(scope, value_shared_memory[write_position] = value);
            gpu!(scope, index_shared_memory[write_position] = index);
        }));
    }

    fn read_from_input(
        scope: &mut Scope,
        input: Variable,
        read_position: Variable,
        i: Variable,
    ) -> Self::Accumulator {
        let value = scope.create_local(input.item());
        gpu!(scope, value = input[read_position]);
        (value, i)
    }

    fn read_from_shared(
        scope: &mut Scope,
        shared_memory: Self::Accumulator,
        read_position: Variable,
    ) -> Self::Accumulator {
        let (value_shared_memory, index_shared_memory) = shared_memory;
        let value = scope.create_local(value_shared_memory.item());
        gpu!(scope, value = value_shared_memory[read_position]);
        let index = scope.create_local(index_shared_memory.item());
        gpu!(scope, index = index_shared_memory[read_position]);
        (value, index)
    }

    fn assign_shared(
        scope: &mut Scope,
        shared_memory: Self::Accumulator,
        output: Variable,
        write_position: Variable,
        _shape_reduce_dim: Variable,
    ) {
        let (_, index_shared_memory) = shared_memory;
        let final_value = scope.create_local(output.item());
        gpu!(scope, final_value = index_shared_memory[0]);
        gpu!(scope, output[write_position] = final_value);
    }
}
