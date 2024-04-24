use crate::{
    codegen::{
        dialect::gpu::{gpu, Elem, Scope, Variable, Visibility},
        Compilation, CompilationInfo, CompilationSettings, EagerHandle, Execution, InputInfo,
        OutputInfo, WorkgroupLaunch,
    },
    element::JitElement,
    gpu::ComputeShader,
    kernel::GpuComputeShaderPhase,
    ops::numeric::empty_device,
    tensor::JitTensor,
    Runtime,
};
use burn_tensor::{ElementConversion, Shape};
use std::{marker::PhantomData, ops::Range};

#[derive(new)]
struct SliceEagerKernel<R: Runtime, E: JitElement> {
    rank: usize,
    _runtime: PhantomData<R>,
    _elem: PhantomData<E>,
}

pub struct SliceComputeShader {
    input: Variable,
    output: Variable,
    rank: usize,
}

impl SliceComputeShader {
    pub fn expand(self, scope: &mut Scope) {
        let input = self.input;
        let output = self.output;
        let id = Variable::Id;

        let offset_input = scope.zero(Elem::UInt);
        let offset_local = scope.create_local(Elem::UInt);

        let stride_input = scope.create_local(Elem::UInt);
        let stride_output = scope.create_local(Elem::UInt);
        let shape_output = scope.create_local(Elem::UInt);
        let range_start = scope.create_local(Elem::UInt);

        for i in 0..self.rank {
            gpu!(scope, stride_input = stride(input, i));
            gpu!(scope, stride_output = stride(output, i));
            gpu!(scope, shape_output = shape(output, i));
            gpu!(
                scope,
                range_start = cast(Variable::GlobalScalar(i as u16, Elem::UInt))
            );

            gpu!(scope, offset_local = id / stride_output);
            gpu!(scope, offset_local = offset_local % shape_output);
            gpu!(scope, offset_local = offset_local + range_start);
            gpu!(scope, offset_local = offset_local * stride_input);

            gpu!(scope, offset_input += offset_local);
        }

        let result = scope.create_local(input.item());
        gpu!(scope, result = input[offset_input]);
        gpu!(scope, output[id] = result);
    }
}

impl<R: Runtime, E: JitElement> GpuComputeShaderPhase for SliceEagerKernel<R, E> {
    fn compile(&self) -> ComputeShader {
        let mut scope = Scope::root();
        let item = E::gpu_elem().into();

        let input = Variable::GlobalInputArray(0, item);
        let output = Variable::GlobalOutputArray(0, item);

        scope.write_global_custom(output);

        SliceComputeShader {
            input,
            output,
            rank: self.rank,
        }
        .expand(&mut scope);

        let input = InputInfo::Array {
            item,
            visibility: Visibility::Read,
        };
        let ranges = InputInfo::Scalar {
            elem: Elem::UInt,
            size: self.rank,
        };
        let output = OutputInfo::Array { item };

        let info = CompilationInfo {
            inputs: vec![input, ranges],
            outputs: vec![output],
            scope,
        };

        let settings = CompilationSettings::default();
        Compilation::new(info).compile(settings)
    }

    fn id(&self) -> String {
        format!("{:?}-rank={:?}", core::any::TypeId::of::<Self>(), self.rank)
    }
}

pub(crate) fn slice<R: Runtime, E: JitElement, const D1: usize, const D2: usize>(
    tensor: JitTensor<R, E, D1>,
    indices: [Range<usize>; D2],
) -> JitTensor<R, E, D1> {
    let mut dims = tensor.shape.dims;
    for i in 0..D2 {
        dims[i] = indices[i].end - indices[i].start;
    }
    let shape_output = Shape::new(dims);
    let output = empty_device(tensor.client.clone(), tensor.device.clone(), shape_output);
    slice_on_output(tensor, output, indices)
}

pub(crate) fn slice_on_output<R: Runtime, E: JitElement, const D1: usize, const D2: usize>(
    tensor: JitTensor<R, E, D1>,
    output: JitTensor<R, E, D1>,
    indices: [Range<usize>; D2],
) -> JitTensor<R, E, D1> {
    let mut scalars: Vec<i32> = Vec::with_capacity(D1);

    for i in 0..D1 {
        let start = indices.get(i).map(|index| index.start).unwrap_or(0);
        scalars.push((start as i32).elem());
    }

    let kernel = SliceEagerKernel::<R, E>::new(D1);

    Execution::start(kernel, tensor.client)
        .inputs(&[EagerHandle::<R>::new(
            &tensor.handle,
            &tensor.strides,
            &tensor.shape.dims,
        )])
        .outputs(&[EagerHandle::new(
            &output.handle,
            &output.strides,
            &output.shape.dims,
        )])
        .with_scalars(&scalars)
        .execute(WorkgroupLaunch::Output { pos: 0 });

    output
}
