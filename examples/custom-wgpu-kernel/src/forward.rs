use crate::FloatTensor;

use super::Backend;
use burn::{
    backend::wgpu::{
        build_info, into_contiguous, kernel_wgsl, FloatElement, GraphicsApi, IntElement,
        JitBackend, JitTensor, Kernel, KernelSource, SourceKernel, SourceTemplate, WgpuRuntime,
        WorkGroup, WorkgroupSize,
    },
    tensor::Shape,
};
use derive_new::new;
use std::marker::PhantomData;

// Source the kernel written in WGSL.
kernel_wgsl!(FusedMatmulAddReluRaw, "./kernel.wgsl");

// Define our kernel type with workgroup information.
#[derive(new, Debug)]
struct FusedMatmulAddRelu<E: FloatElement> {
    workgroup_size: WorkgroupSize,
    _elem: PhantomData<E>,
}

// Implement the dynamic kernel trait for our kernel type.
impl<E: FloatElement> KernelSource for FusedMatmulAddRelu<E> {
    fn source(&self) -> SourceTemplate {
        // Extend our raw kernel with workgroup size information using the
        // `SourceTemplate` trait.
        FusedMatmulAddReluRaw::new()
            .source()
            .register("workgroup_size_x", self.workgroup_size.x.to_string())
            .register("workgroup_size_y", self.workgroup_size.y.to_string())
            .register("elem", E::type_name())
            .register("int", "i32")
    }
}

/// Implement our custom backend trait for the existing backend `WgpuBackend`.
impl<G: GraphicsApi, F: FloatElement, I: IntElement> Backend for JitBackend<WgpuRuntime<G, F, I>> {
    fn fused_matmul_add_relu<const D: usize>(
        lhs: FloatTensor<Self, D>,
        rhs: FloatTensor<Self, D>,
        bias: FloatTensor<Self, D>,
    ) -> FloatTensor<Self, D> {
        // Define workgroup size, hardcoded for simplicity.
        let workgroup_size = WorkgroupSize { x: 16, y: 16, z: 1 };

        // Specify the size of a workgroup for this kernel
        lhs.assert_is_on_same_device(&rhs);
        lhs.assert_is_on_same_device(&bias);

        // For simplicity, make sure each tensor is continuous.
        let lhs = into_contiguous(lhs);
        let rhs = into_contiguous(rhs);
        let bias = into_contiguous(bias);

        // Get the matmul relevant shapes.
        let num_rows = lhs.shape.dims[D - 2];
        let num_cols = rhs.shape.dims[D - 1];

        // Compute shape of output, while tracking number of batches.
        let mut num_batches = 1;
        let mut shape_out = [0; D];
        for i in shape_out.into_iter().take(D - 2) {
            shape_out[i] = usize::max(lhs.shape.dims[i], rhs.shape.dims[i]);
            num_batches *= shape_out[i];
        }
        shape_out[D - 2] = num_rows;
        shape_out[D - 1] = num_cols;
        let shape_out = Shape::new(shape_out);

        // Create a buffer for the output tensor.
        let buffer = lhs
            .client
            .empty(shape_out.num_elements() * core::mem::size_of::<F>());

        // Create the output tensor primitive.
        let output = JitTensor::new(lhs.client.clone(), lhs.device.clone(), shape_out, buffer);

        // Create the kernel.
        let kernel = FusedMatmulAddRelu::<F>::new(workgroup_size);

        // Build info buffer with tensor information needed by the kernel, such as shapes and strides.
        let info = build_info(&[&lhs, &rhs, &output]);
        let info_handle = lhs.client.create(bytemuck::cast_slice(&info));

        // Declare the wgsl workgroup with the number of blocks in x, y and z.
        let blocks_needed_in_x = f32::ceil(num_rows as f32 / workgroup_size.x as f32) as u32;
        let blocks_needed_in_y = f32::ceil(num_cols as f32 / workgroup_size.y as f32) as u32;
        let workgroup = WorkGroup::new(blocks_needed_in_x, blocks_needed_in_y, num_batches as u32);

        // Execute lazily the kernel with the launch information and the given buffers.
        lhs.client.execute(
            Kernel::Custom(Box::new(SourceKernel::new(
                kernel,
                workgroup,
                workgroup_size,
            ))),
            vec![
                lhs.handle.binding(),
                rhs.handle.binding(),
                bias.handle.binding(),
                output.handle.clone().binding(),
                info_handle.binding(),
            ],
        );

        // Return the output tensor.
        output
    }
}
