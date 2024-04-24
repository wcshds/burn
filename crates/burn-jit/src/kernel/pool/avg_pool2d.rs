use crate::{
    codegen::{dialect::gpu::Variable, EagerHandle, Execution, WorkgroupLaunch},
    element::JitElement,
    gpu::{gpu, Elem, Item, Scope},
    ops::numeric::empty_device,
    tensor::JitTensor,
    Runtime,
};
use burn_tensor::{ops::conv::calculate_pool_output_size, Shape};
use std::fmt::Debug;

use super::{Pool2dEagerKernel, PoolStrategy};

#[derive(new, Debug, Clone)]
struct AvgPool {
    kernel_size: [usize; 2],
    count_include_pad: bool,
}

impl PoolStrategy for AvgPool {
    type Accumulator = (Variable, Variable);

    fn initialize(&self, scope: &mut Scope, item: Item) -> Self::Accumulator {
        let sum = scope.create_local(item);
        let count = scope.create_local(Elem::UInt);
        if self.count_include_pad {
            let kernel_size: Variable = (self.kernel_size[0] * self.kernel_size[1]).into();
            gpu!(scope, count = kernel_size);
        } else {
            let zero: Variable = 0u32.into();
            gpu!(scope, count = zero);
        }
        (sum, count)
    }

    fn process_result(
        &self,
        scope: &mut Scope,
        accumulator: Self::Accumulator,
        result: Variable,
        _idx: Variable,
    ) -> Self::Accumulator {
        let (sum, count) = accumulator;
        if !self.count_include_pad {
            let one: Variable = 1u32.into();
            gpu!(scope, count += one);
        }
        gpu!(scope, sum += result);
        (sum, count)
    }

    fn assign(
        &self,
        scope: &mut Scope,
        id: Variable,
        output: Variable,
        _indices: Option<Variable>,
        accumulator: Self::Accumulator,
    ) {
        let (sum, count) = accumulator;
        let avg = scope.create_local(output.item());
        let count_float = scope.create_local(output.item());
        gpu!(scope, count_float = cast(count));
        gpu!(scope, avg = sum / count_float);
        gpu!(scope, output[id] = avg);
    }

    fn with_indices() -> bool {
        false
    }
}

pub(crate) fn avg_pool2d<R: Runtime, E: JitElement>(
    x: JitTensor<R, E, 4>,
    kernel_size: [usize; 2],
    stride: [usize; 2],
    padding: [usize; 2],
    count_include_pad: bool,
) -> JitTensor<R, E, 4> {
    let [batch_size, channels, _, _] = x.shape.dims;
    let dilation = 1;

    let size_0 = calculate_pool_output_size(
        kernel_size[0],
        stride[0],
        padding[0],
        dilation,
        x.shape.dims[2],
    );
    let size_1 = calculate_pool_output_size(
        kernel_size[1],
        stride[1],
        padding[1],
        dilation,
        x.shape.dims[3],
    );

    let shape_out = Shape::new([batch_size, channels, size_0, size_1]);
    let output = empty_device(x.client.clone(), x.device.clone(), shape_out);

    let pool_strategy = AvgPool::new(kernel_size, count_include_pad);
    let kernel = Pool2dEagerKernel::<AvgPool, R, E>::new(kernel_size, pool_strategy);

    Execution::start(kernel, x.client)
        .inputs(&[EagerHandle::<R>::new(&x.handle, &x.strides, &x.shape.dims)])
        .outputs(&[EagerHandle::new(
            &output.handle,
            &output.strides,
            &output.shape.dims,
        )])
        .with_scalars(&[
            stride[0] as u32,
            stride[1] as u32,
            dilation as u32,
            dilation as u32,
            padding[0] as u32,
            padding[1] as u32,
        ])
        .execute(WorkgroupLaunch::Output { pos: 0 });

    output
}
