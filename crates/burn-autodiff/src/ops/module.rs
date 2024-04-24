use crate::checkpoint::base::Checkpointer;
use crate::checkpoint::strategy::CheckpointStrategy;
use crate::grads::Gradients;
use crate::graph::NodeID;
use crate::ops::{unary, Backward, Ops};
use crate::tensor::AutodiffTensor;
use crate::Autodiff;

use burn_tensor::backend::Backend;
use burn_tensor::ops::*;

use super::OpsKind;

impl<B: Backend, C: CheckpointStrategy> ModuleOps<Autodiff<B, C>> for Autodiff<B, C> {
    fn embedding(weights: AutodiffTensor<B, 2>, indices: IntTensor<B, 2>) -> AutodiffTensor<B, 3> {
        #[derive(Debug)]
        struct Embedding;

        impl<B: Backend> Backward<B, 3, 1> for Embedding {
            type State = (B::FloatTensorPrimitive<2>, IntTensor<B, 2>);

            fn backward(
                self,
                ops: Ops<Self::State, 1>,
                grads: &mut Gradients,
                _checkpointer: &mut Checkpointer,
            ) {
                let (weights, indices) = ops.state;

                unary::<B, 3, 2, _>(ops.parents, ops.node, grads, |grad| {
                    B::embedding_backward(weights, grad, indices)
                });
            }
        }

        match Embedding
            .prepare::<C>([weights.node])
            .compute_bound()
            .stateful()
        {
            OpsKind::Tracked(prep) => prep.finish(
                (weights.primitive.clone(), indices.clone()),
                B::embedding(weights.primitive, indices),
            ),
            OpsKind::UnTracked(prep) => prep.finish(B::embedding(weights.primitive, indices)),
        }
    }

    fn embedding_backward(
        _weights: AutodiffTensor<B, 2>,
        _output: AutodiffTensor<B, 3>,
        _indices: IntTensor<B, 2>,
    ) -> AutodiffTensor<B, 2> {
        panic!("Can't differentiate embedding backward.");
    }

    fn conv2d(
        x: AutodiffTensor<B, 4>,
        weight: AutodiffTensor<B, 4>,
        bias: Option<AutodiffTensor<B, 1>>,
        options: ConvOptions<2>,
    ) -> AutodiffTensor<B, 4> {
        #[derive(Debug)]
        struct Conv2DWithBias;
        #[derive(Debug)]
        struct Conv2DNoBias;

        impl<B: Backend> Backward<B, 4, 3> for Conv2DWithBias {
            type State = (NodeID, NodeID, NodeID, ConvOptions<2>);

            fn backward(
                self,
                ops: Ops<Self::State, 3>,
                grads: &mut Gradients,
                checkpointer: &mut Checkpointer,
            ) {
                let [node_x, node_weight, node_bias] = ops.parents;
                let grad = grads.consume::<B, 4>(&ops.node);

                let (x_state, weight_state, bias_state, options) = ops.state;
                let x = checkpointer.retrieve_node_output(x_state);
                let weight = checkpointer.retrieve_node_output(weight_state);
                let bias = Some(checkpointer.retrieve_node_output(bias_state));

                let backward = B::conv2d_backward(x, weight, bias, grad, options);

                if let Some(node) = node_x {
                    grads.register::<B, 4>(node.id, backward.x_grad)
                }
                if let Some(node) = node_weight {
                    grads.register::<B, 4>(node.id, backward.weights_grad)
                }
                if let Some(node) = node_bias {
                    grads.register::<B, 1>(node.id, backward.bias_grad.unwrap())
                }
            }
        }

        impl<B: Backend> Backward<B, 4, 2> for Conv2DNoBias {
            type State = (NodeID, NodeID, ConvOptions<2>);

            fn backward(
                self,
                ops: Ops<Self::State, 2>,
                grads: &mut Gradients,
                checkpointer: &mut Checkpointer,
            ) {
                let [node_x, node_weight] = ops.parents;
                let grad = grads.consume::<B, 4>(&ops.node);

                let (x_state, weight_state, options) = ops.state;
                let x = checkpointer.retrieve_node_output(x_state);
                let weight = checkpointer.retrieve_node_output(weight_state);

                let backward = B::conv2d_backward(x, weight, None, grad, options);

                if let Some(node) = node_x {
                    grads.register::<B, 4>(node.id, backward.x_grad)
                }
                if let Some(node) = node_weight {
                    grads.register::<B, 4>(node.id, backward.weights_grad)
                }
            }
        }

        match bias {
            Some(bias) => match Conv2DWithBias
                .prepare::<C>([x.node.clone(), weight.node.clone(), bias.node.clone()])
                .compute_bound()
                .stateful()
            {
                OpsKind::Tracked(mut prep) => {
                    let x_state = prep.checkpoint(&x);
                    let weight_state = prep.checkpoint(&weight);
                    let bias_state = prep.checkpoint(&bias);
                    prep.finish(
                        (x_state, weight_state, bias_state, options.clone()),
                        B::conv2d(x.primitive, weight.primitive, Some(bias.primitive), options),
                    )
                }
                OpsKind::UnTracked(prep) => prep.finish(B::conv2d(
                    x.primitive,
                    weight.primitive,
                    Some(bias.primitive),
                    options,
                )),
            },
            None => match Conv2DNoBias
                .prepare::<C>([x.node.clone(), weight.node.clone()])
                .compute_bound()
                .stateful()
            {
                OpsKind::Tracked(mut prep) => {
                    let x_state = prep.checkpoint(&x);
                    let weight_state = prep.checkpoint(&weight);
                    prep.finish(
                        (x_state, weight_state, options.clone()),
                        B::conv2d(x.primitive, weight.primitive, None, options),
                    )
                }

                OpsKind::UnTracked(prep) => {
                    prep.finish(B::conv2d(x.primitive, weight.primitive, None, options))
                }
            },
        }
    }

    fn conv_transpose2d(
        x: AutodiffTensor<B, 4>,
        weight: AutodiffTensor<B, 4>,
        bias: Option<AutodiffTensor<B, 1>>,
        options: ConvTransposeOptions<2>,
    ) -> AutodiffTensor<B, 4> {
        #[derive(Debug)]
        struct ConvTranspose2DWithBias;
        #[derive(Debug)]
        struct ConvTranspose2DNoBias;

        impl<B: Backend> Backward<B, 4, 3> for ConvTranspose2DWithBias {
            type State = (NodeID, NodeID, NodeID, ConvTransposeOptions<2>);

            fn backward(
                self,
                ops: Ops<Self::State, 3>,
                grads: &mut Gradients,
                checkpointer: &mut Checkpointer,
            ) {
                let [node_x, node_weight, node_bias] = ops.parents;
                let grad = grads.consume::<B, 4>(&ops.node);

                let (x_state, weight_state, bias_state, options) = ops.state;
                let x = checkpointer.retrieve_node_output(x_state);
                let weight = checkpointer.retrieve_node_output(weight_state);
                let bias = Some(checkpointer.retrieve_node_output(bias_state));

                let backward = B::conv_transpose2d_backward(x, weight, bias, grad, options);

                if let Some(node) = node_x {
                    grads.register::<B, 4>(node.id, backward.x_grad)
                }
                if let Some(node) = node_weight {
                    grads.register::<B, 4>(node.id, backward.weights_grad)
                }
                if let Some(node) = node_bias {
                    grads.register::<B, 1>(node.id, backward.bias_grad.unwrap())
                }
            }
        }

        impl<B: Backend> Backward<B, 4, 2> for ConvTranspose2DNoBias {
            type State = (NodeID, NodeID, ConvTransposeOptions<2>);

            fn backward(
                self,
                ops: Ops<Self::State, 2>,
                grads: &mut Gradients,
                checkpointer: &mut Checkpointer,
            ) {
                let [node_x, node_weight] = ops.parents;
                let grad = grads.consume::<B, 4>(&ops.node);

                let (x_state, weight_state, options) = ops.state;
                let x = checkpointer.retrieve_node_output(x_state);
                let weight = checkpointer.retrieve_node_output(weight_state);

                let backward = B::conv_transpose2d_backward(x, weight, None, grad, options);

                if let Some(node) = node_x {
                    grads.register::<B, 4>(node.id, backward.x_grad)
                }
                if let Some(node) = node_weight {
                    grads.register::<B, 4>(node.id, backward.weights_grad)
                }
            }
        }

        match bias {
            Some(bias) => match ConvTranspose2DWithBias
                .prepare::<C>([x.node.clone(), weight.node.clone(), bias.node.clone()])
                .compute_bound()
                .stateful()
            {
                OpsKind::Tracked(mut prep) => {
                    let x_state = prep.checkpoint(&x);
                    let weight_state = prep.checkpoint(&weight);
                    let bias_state = prep.checkpoint(&bias);

                    prep.finish(
                        (x_state, weight_state, bias_state, options.clone()),
                        B::conv_transpose2d(
                            x.primitive,
                            weight.primitive,
                            Some(bias.primitive),
                            options,
                        ),
                    )
                }
                OpsKind::UnTracked(prep) => prep.finish(B::conv_transpose2d(
                    x.primitive,
                    weight.primitive,
                    Some(bias.primitive),
                    options,
                )),
            },
            None => match ConvTranspose2DNoBias
                .prepare::<C>([x.node.clone(), weight.node.clone()])
                .compute_bound()
                .stateful()
            {
                OpsKind::Tracked(mut prep) => {
                    let x_state = prep.checkpoint(&x);
                    let weight_state = prep.checkpoint(&weight);

                    prep.finish(
                        (x_state, weight_state, options.clone()),
                        B::conv_transpose2d(x.primitive, weight.primitive, None, options),
                    )
                }
                OpsKind::UnTracked(prep) => prep.finish(B::conv_transpose2d(
                    x.primitive,
                    weight.primitive,
                    None,
                    options,
                )),
            },
        }
    }

    fn conv1d(
        x: AutodiffTensor<B, 3>,
        weight: AutodiffTensor<B, 3>,
        bias: Option<AutodiffTensor<B, 1>>,
        options: ConvOptions<1>,
    ) -> AutodiffTensor<B, 3> {
        #[derive(Debug)]
        struct Conv1DWithBias;
        #[derive(Debug)]
        struct Conv1DNoBias;

        impl<B: Backend> Backward<B, 3, 3> for Conv1DWithBias {
            type State = (NodeID, NodeID, NodeID, ConvOptions<1>);

            fn backward(
                self,
                ops: Ops<Self::State, 3>,
                grads: &mut Gradients,
                checkpointer: &mut Checkpointer,
            ) {
                let [node_x, node_weight, node_bias] = ops.parents;
                let grad = grads.consume::<B, 3>(&ops.node);

                let (x_state, weight_state, bias_state, options) = ops.state;
                let x = checkpointer.retrieve_node_output(x_state);
                let weight = checkpointer.retrieve_node_output(weight_state);
                let bias = Some(checkpointer.retrieve_node_output(bias_state));

                let backward = B::conv1d_backward(x, weight, bias, grad, options);

                if let Some(node) = node_x {
                    grads.register::<B, 3>(node.id, backward.x_grad)
                }
                if let Some(node) = node_weight {
                    grads.register::<B, 3>(node.id, backward.weights_grad)
                }
                if let Some(node) = node_bias {
                    grads.register::<B, 1>(node.id, backward.bias_grad.unwrap())
                }
            }
        }

        impl<B: Backend> Backward<B, 3, 2> for Conv1DNoBias {
            type State = (NodeID, NodeID, ConvOptions<1>);

            fn backward(
                self,
                ops: Ops<Self::State, 2>,
                grads: &mut Gradients,
                checkpointer: &mut Checkpointer,
            ) {
                let [node_x, node_weight] = ops.parents;
                let grad = grads.consume::<B, 3>(&ops.node);

                let (x_state, weight_state, options) = ops.state;
                let x = checkpointer.retrieve_node_output(x_state);
                let weight = checkpointer.retrieve_node_output(weight_state);

                let backward = B::conv1d_backward(x, weight, None, grad, options);

                if let Some(node) = node_x {
                    grads.register::<B, 3>(node.id, backward.x_grad)
                }
                if let Some(node) = node_weight {
                    grads.register::<B, 3>(node.id, backward.weights_grad)
                }
            }
        }
        match bias {
            Some(bias) => match Conv1DWithBias
                .prepare::<C>([x.node.clone(), weight.node.clone(), bias.node.clone()])
                .compute_bound()
                .stateful()
            {
                OpsKind::Tracked(mut prep) => {
                    let x_state = prep.checkpoint(&x);
                    let weight_state = prep.checkpoint(&weight);
                    let bias_state = prep.checkpoint(&bias);
                    prep.finish(
                        (x_state, weight_state, bias_state, options.clone()),
                        B::conv1d(x.primitive, weight.primitive, Some(bias.primitive), options),
                    )
                }
                OpsKind::UnTracked(prep) => prep.finish(B::conv1d(
                    x.primitive,
                    weight.primitive,
                    Some(bias.primitive),
                    options,
                )),
            },
            None => match Conv1DNoBias
                .prepare::<C>([x.node.clone(), weight.node.clone()])
                .compute_bound()
                .stateful()
            {
                OpsKind::Tracked(mut prep) => {
                    let x_state = prep.checkpoint(&x);
                    let weight_state = prep.checkpoint(&weight);
                    prep.finish(
                        (x_state, weight_state, options.clone()),
                        B::conv1d(x.primitive, weight.primitive, None, options),
                    )
                }
                OpsKind::UnTracked(prep) => {
                    prep.finish(B::conv1d(x.primitive, weight.primitive, None, options))
                }
            },
        }
    }

    fn conv_transpose1d(
        x: AutodiffTensor<B, 3>,
        weight: AutodiffTensor<B, 3>,
        bias: Option<AutodiffTensor<B, 1>>,
        options: ConvTransposeOptions<1>,
    ) -> AutodiffTensor<B, 3> {
        #[derive(Debug)]
        struct ConvTranspose1DWithBias;
        #[derive(Debug)]
        struct ConvTranspose1DNoBias;

        impl<B: Backend> Backward<B, 3, 3> for ConvTranspose1DWithBias {
            type State = (NodeID, NodeID, NodeID, ConvTransposeOptions<1>);

            fn backward(
                self,
                ops: Ops<Self::State, 3>,
                grads: &mut Gradients,
                checkpointer: &mut Checkpointer,
            ) {
                let [node_x, node_weight, node_bias] = ops.parents;
                let grad = grads.consume::<B, 3>(&ops.node);

                let (x_state, weight_state, bias_state, options) = ops.state;
                let x = checkpointer.retrieve_node_output(x_state);
                let weight = checkpointer.retrieve_node_output(weight_state);
                let bias = Some(checkpointer.retrieve_node_output(bias_state));

                let backward = B::conv_transpose1d_backward(x, weight, bias, grad, options);

                if let Some(node) = node_x {
                    grads.register::<B, 3>(node.id, backward.x_grad)
                }
                if let Some(node) = node_weight {
                    grads.register::<B, 3>(node.id, backward.weights_grad)
                }
                if let Some(node) = node_bias {
                    grads.register::<B, 1>(node.id, backward.bias_grad.unwrap())
                }
            }
        }

        impl<B: Backend> Backward<B, 3, 2> for ConvTranspose1DNoBias {
            type State = (NodeID, NodeID, ConvTransposeOptions<1>);

            fn backward(
                self,
                ops: Ops<Self::State, 2>,
                grads: &mut Gradients,
                checkpointer: &mut Checkpointer,
            ) {
                let [node_x, node_weight] = ops.parents;
                let grad = grads.consume::<B, 3>(&ops.node);

                let (x_state, weight_state, options) = ops.state;
                let x = checkpointer.retrieve_node_output(x_state);
                let weight = checkpointer.retrieve_node_output(weight_state);

                let backward = B::conv_transpose1d_backward(x, weight, None, grad, options);

                if let Some(node) = node_x {
                    grads.register::<B, 3>(node.id, backward.x_grad)
                }
                if let Some(node) = node_weight {
                    grads.register::<B, 3>(node.id, backward.weights_grad)
                }
            }
        }

        match bias {
            Some(bias) => match ConvTranspose1DWithBias
                .prepare::<C>([x.node.clone(), weight.node.clone(), bias.node.clone()])
                .compute_bound()
                .stateful()
            {
                OpsKind::Tracked(mut prep) => {
                    let x_state = prep.checkpoint(&x);
                    let weight_state = prep.checkpoint(&weight);
                    let bias_state = prep.checkpoint(&bias);
                    prep.finish(
                        (x_state, weight_state, bias_state, options.clone()),
                        B::conv_transpose1d(
                            x.primitive,
                            weight.primitive,
                            Some(bias.primitive),
                            options,
                        ),
                    )
                }
                OpsKind::UnTracked(prep) => prep.finish(B::conv_transpose1d(
                    x.primitive,
                    weight.primitive,
                    Some(bias.primitive),
                    options,
                )),
            },
            None => match ConvTranspose1DNoBias
                .prepare::<C>([x.node.clone(), weight.node.clone()])
                .compute_bound()
                .stateful()
            {
                OpsKind::Tracked(mut prep) => {
                    let x_state = prep.checkpoint(&x);
                    let weight_state = prep.checkpoint(&weight);
                    prep.finish(
                        (x_state, weight_state, options.clone()),
                        B::conv_transpose1d(x.primitive, weight.primitive, None, options),
                    )
                }
                OpsKind::UnTracked(prep) => prep.finish(B::conv_transpose1d(
                    x.primitive,
                    weight.primitive,
                    None,
                    options,
                )),
            },
        }
    }

    // TODO: Support a custom unfold4d operation by overriding the default implementation.
    //
    // We don't override it now because the fold operation isn't available for the backward pass.
    // This implies that when autodiff is enabled, custom unfold operations defined by backends
    // won't be used. Instead, the conv2d operation with custom weights matrix will be used.
    // Therefore, the conv2d backward pass will be used for the unfold4d backward pass.
    //
    // fn unfold4d(
    //     x: AutodiffTensor<B, 4>,
    //     kernel_size: [usize; 2],
    //     options: UnfoldOptions,
    // ) -> AutodiffTensor<B, 3> {
    //     todo!()
    // }

    fn avg_pool1d(
        x: AutodiffTensor<B, 3>,
        kernel_size: usize,
        stride: usize,
        padding: usize,
        count_include_pad: bool,
    ) -> AutodiffTensor<B, 3> {
        #[derive(Debug)]
        struct AvgPool1D;

        impl<B: Backend> Backward<B, 3, 1> for AvgPool1D {
            type State = (NodeID, usize, usize, usize, bool);

            fn backward(
                self,
                ops: Ops<Self::State, 1>,
                grads: &mut Gradients,
                checkpointer: &mut Checkpointer,
            ) {
                let [node_parent] = ops.parents;
                let grad = grads.consume::<B, 3>(&ops.node);
                let (x_state, kernel_size, stride, padding, count_include_pad) = ops.state;
                let x = checkpointer.retrieve_node_output(x_state);

                if let Some(node) = node_parent {
                    let grad = B::avg_pool1d_backward(
                        x,
                        grad,
                        kernel_size,
                        stride,
                        padding,
                        count_include_pad,
                    );
                    grads.register::<B, 3>(node.id, grad);
                }
            }
        }

        match AvgPool1D
            .prepare::<C>([x.node.clone()])
            .compute_bound()
            .stateful()
        {
            OpsKind::Tracked(mut prep) => {
                let x_state = prep.checkpoint(&x);
                prep.finish(
                    (x_state, kernel_size, stride, padding, count_include_pad),
                    B::avg_pool1d(
                        x.primitive.clone(),
                        kernel_size,
                        stride,
                        padding,
                        count_include_pad,
                    ),
                )
            }
            OpsKind::UnTracked(prep) => prep.finish(B::avg_pool1d(
                x.primitive,
                kernel_size,
                stride,
                padding,
                count_include_pad,
            )),
        }
    }

    fn avg_pool2d(
        x: AutodiffTensor<B, 4>,
        kernel_size: [usize; 2],
        stride: [usize; 2],
        padding: [usize; 2],
        count_include_pad: bool,
    ) -> AutodiffTensor<B, 4> {
        #[derive(Debug)]
        struct AvgPool2D;

        impl<B: Backend> Backward<B, 4, 1> for AvgPool2D {
            type State = (NodeID, [usize; 2], [usize; 2], [usize; 2], bool);

            fn backward(
                self,
                ops: Ops<Self::State, 1>,
                grads: &mut Gradients,
                checkpointer: &mut Checkpointer,
            ) {
                let [node_parent] = ops.parents;
                let grad = grads.consume::<B, 4>(&ops.node);
                let (x_state, kernel_size, stride, padding, count_include_pad) = ops.state;
                let x = checkpointer.retrieve_node_output(x_state);

                if let Some(node) = node_parent {
                    let grad = B::avg_pool2d_backward(
                        x,
                        grad,
                        kernel_size,
                        stride,
                        padding,
                        count_include_pad,
                    );
                    grads.register::<B, 4>(node.id, grad);
                }
            }
        }

        match AvgPool2D
            .prepare::<C>([x.node.clone()])
            .compute_bound()
            .stateful()
        {
            OpsKind::Tracked(mut prep) => {
                let x_state = prep.checkpoint(&x);
                prep.finish(
                    (x_state, kernel_size, stride, padding, count_include_pad),
                    B::avg_pool2d(
                        x.primitive.clone(),
                        kernel_size,
                        stride,
                        padding,
                        count_include_pad,
                    ),
                )
            }
            OpsKind::UnTracked(prep) => prep.finish(B::avg_pool2d(
                x.primitive,
                kernel_size,
                stride,
                padding,
                count_include_pad,
            )),
        }
    }

    fn avg_pool2d_backward(
        _x: AutodiffTensor<B, 4>,
        _grad: AutodiffTensor<B, 4>,
        _kernel_size: [usize; 2],
        _stride: [usize; 2],
        _padding: [usize; 2],
        _count_include_pad: bool,
    ) -> AutodiffTensor<B, 4> {
        panic!("Can't differentiate avg pool 2d backward.");
    }

    fn max_pool1d(
        x: AutodiffTensor<B, 3>,
        kernel_size: usize,
        stride: usize,
        padding: usize,
        dilation: usize,
    ) -> AutodiffTensor<B, 3> {
        match MaxPool1D
            .prepare::<C>([x.node.clone()])
            .compute_bound()
            .stateful()
        {
            OpsKind::Tracked(mut prep) => {
                let x_state = prep.checkpoint(&x);
                let output =
                    B::max_pool1d_with_indices(x.primitive, kernel_size, stride, padding, dilation);
                prep.finish(
                    (
                        x_state,
                        output.indices,
                        kernel_size,
                        stride,
                        padding,
                        dilation,
                    ),
                    output.output,
                )
            }
            OpsKind::UnTracked(prep) => prep.finish(B::max_pool1d(
                x.primitive,
                kernel_size,
                stride,
                padding,
                dilation,
            )),
        }
    }

    fn max_pool1d_with_indices(
        x: AutodiffTensor<B, 3>,
        kernel_size: usize,
        stride: usize,
        padding: usize,
        dilation: usize,
    ) -> MaxPool1dWithIndices<Self> {
        match MaxPool1D
            .prepare::<C>([x.node.clone()])
            .compute_bound()
            .stateful()
        {
            OpsKind::Tracked(mut prep) => {
                let x_state = prep.checkpoint(&x);
                let output =
                    B::max_pool1d_with_indices(x.primitive, kernel_size, stride, padding, dilation);

                let output_tensor = prep.finish(
                    (
                        x_state,
                        output.indices.clone(),
                        kernel_size,
                        stride,
                        padding,
                        dilation,
                    ),
                    output.output,
                );

                MaxPool1dWithIndices::new(output_tensor, output.indices)
            }
            OpsKind::UnTracked(prep) => {
                let output =
                    B::max_pool1d_with_indices(x.primitive, kernel_size, stride, padding, dilation);
                let output_tensor = prep.finish(output.output);

                MaxPool1dWithIndices::new(output_tensor, output.indices)
            }
        }
    }

    fn max_pool1d_with_indices_backward(
        x: AutodiffTensor<B, 3>,
        kernel_size: usize,
        stride: usize,
        padding: usize,
        dilation: usize,
        output_grad: AutodiffTensor<B, 3>,
        indices: IntTensor<B, 3>,
    ) -> MaxPool1dBackward<Self> {
        let output = B::max_pool1d_with_indices_backward(
            x.primitive,
            kernel_size,
            stride,
            padding,
            dilation,
            output_grad.primitive,
            indices,
        );
        MaxPool1dBackward::new(AutodiffTensor::new(output.x_grad))
    }

    fn max_pool2d(
        x: AutodiffTensor<B, 4>,
        kernel_size: [usize; 2],
        stride: [usize; 2],
        padding: [usize; 2],
        dilation: [usize; 2],
    ) -> AutodiffTensor<B, 4> {
        match MaxPool2D
            .prepare::<C>([x.node.clone()])
            .compute_bound()
            .stateful()
        {
            OpsKind::Tracked(mut prep) => {
                let x_state = prep.checkpoint(&x);
                let output =
                    B::max_pool2d_with_indices(x.primitive, kernel_size, stride, padding, dilation);
                prep.finish(
                    (
                        x_state,
                        output.indices,
                        kernel_size,
                        stride,
                        padding,
                        dilation,
                    ),
                    output.output,
                )
            }
            OpsKind::UnTracked(prep) => prep.finish(B::max_pool2d(
                x.primitive,
                kernel_size,
                stride,
                padding,
                dilation,
            )),
        }
    }

    fn max_pool2d_with_indices(
        x: AutodiffTensor<B, 4>,
        kernel_size: [usize; 2],
        stride: [usize; 2],
        padding: [usize; 2],
        dilation: [usize; 2],
    ) -> MaxPool2dWithIndices<Self> {
        match MaxPool2D
            .prepare::<C>([x.node.clone()])
            .compute_bound()
            .stateful()
        {
            OpsKind::Tracked(mut prep) => {
                let x_state = prep.checkpoint(&x);

                let output =
                    B::max_pool2d_with_indices(x.primitive, kernel_size, stride, padding, dilation);

                let output_tensor = prep.finish(
                    (
                        x_state,
                        output.indices.clone(),
                        kernel_size,
                        stride,
                        padding,
                        dilation,
                    ),
                    output.output,
                );

                MaxPool2dWithIndices::new(output_tensor, output.indices)
            }
            OpsKind::UnTracked(prep) => {
                let output =
                    B::max_pool2d_with_indices(x.primitive, kernel_size, stride, padding, dilation);
                let output_tensor = prep.finish(output.output);

                MaxPool2dWithIndices::new(output_tensor, output.indices)
            }
        }
    }

    fn max_pool2d_with_indices_backward(
        _x: AutodiffTensor<B, 4>,
        _kernel_size: [usize; 2],
        _stride: [usize; 2],
        _padding: [usize; 2],
        _dilation: [usize; 2],
        _output_grad: AutodiffTensor<B, 4>,
        _indices: IntTensor<B, 4>,
    ) -> MaxPool2dBackward<Self> {
        panic!("Can't differentiate max pool2d with indices backward.");
    }
    fn adaptive_avg_pool1d(x: AutodiffTensor<B, 3>, output_size: usize) -> AutodiffTensor<B, 3> {
        #[derive(Debug)]
        struct AdaptiveAvgPool1D;

        impl<B: Backend> Backward<B, 3, 1> for AdaptiveAvgPool1D {
            type State = NodeID;

            fn backward(
                self,
                ops: Ops<Self::State, 1>,
                grads: &mut Gradients,
                checkpointer: &mut Checkpointer,
            ) {
                let [node_parent] = ops.parents;
                let grad = grads.consume::<B, 3>(&ops.node);
                let state = checkpointer.retrieve_node_output(ops.state);

                if let Some(node) = node_parent {
                    let grad = B::adaptive_avg_pool1d_backward(state, grad);
                    grads.register::<B, 3>(node.id, grad);
                }
            }
        }

        match AdaptiveAvgPool1D
            .prepare::<C>([x.node.clone()])
            .compute_bound()
            .stateful()
        {
            OpsKind::Tracked(mut prep) => {
                let x_state = prep.checkpoint(&x);
                prep.finish(x_state, B::adaptive_avg_pool1d(x.primitive, output_size))
            }
            OpsKind::UnTracked(prep) => {
                prep.finish(B::adaptive_avg_pool1d(x.primitive, output_size))
            }
        }
    }

    fn adaptive_avg_pool2d(
        x: AutodiffTensor<B, 4>,
        output_size: [usize; 2],
    ) -> AutodiffTensor<B, 4> {
        #[derive(Debug)]
        struct AdaptiveAvgPool2D;

        impl<B: Backend> Backward<B, 4, 1> for AdaptiveAvgPool2D {
            type State = NodeID;

            fn backward(
                self,
                ops: Ops<Self::State, 1>,
                grads: &mut Gradients,
                checkpointer: &mut Checkpointer,
            ) {
                let [node_parent] = ops.parents;
                let grad = grads.consume::<B, 4>(&ops.node);
                let state = checkpointer.retrieve_node_output(ops.state);

                if let Some(node) = node_parent {
                    let grad = B::adaptive_avg_pool2d_backward(state, grad);
                    grads.register::<B, 4>(node.id, grad);
                }
            }
        }

        match AdaptiveAvgPool2D
            .prepare::<C>([x.node.clone()])
            .compute_bound()
            .stateful()
        {
            OpsKind::Tracked(mut prep) => {
                let x_state = prep.checkpoint(&x);
                prep.finish(x_state, B::adaptive_avg_pool2d(x.primitive, output_size))
            }
            OpsKind::UnTracked(prep) => {
                prep.finish(B::adaptive_avg_pool2d(x.primitive, output_size))
            }
        }
    }

    fn adaptive_avg_pool2d_backward(
        _x: AutodiffTensor<B, 4>,
        _grad: AutodiffTensor<B, 4>,
    ) -> <Autodiff<B> as Backend>::FloatTensorPrimitive<4> {
        panic!("Can't differentiate adaptive avg pool2d backward.");
    }

    fn interpolate(
        x: AutodiffTensor<B, 4>,
        output_size: [usize; 2],
        options: InterpolateOptions,
    ) -> AutodiffTensor<B, 4> {
        #[derive(Debug)]
        struct Interpolate;
        impl<B: Backend> Backward<B, 4, 1> for Interpolate {
            type State = (NodeID, [usize; 2], InterpolateOptions);

            fn backward(
                self,
                ops: Ops<Self::State, 1>,
                grads: &mut Gradients,
                checkpointer: &mut Checkpointer,
            ) {
                let [node_parent] = ops.parents;
                let grad = grads.consume::<B, 4>(&ops.node);

                let (x_state, output_size, options) = ops.state;
                let state = checkpointer.retrieve_node_output(x_state);

                if let Some(node) = node_parent {
                    let grad = B::interpolate_backward(state, grad, output_size, options);
                    grads.register::<B, 4>(node.id, grad);
                }
            }
        }

        match Interpolate
            .prepare::<C>([x.node.clone()])
            .compute_bound()
            .stateful()
        {
            OpsKind::Tracked(mut prep) => {
                let x_state = prep.checkpoint(&x);
                let output = B::interpolate(x.primitive.clone(), output_size, options.clone());
                prep.finish((x_state, output_size, options), output)
            }
            OpsKind::UnTracked(prep) => {
                prep.finish(B::interpolate(x.primitive, output_size, options))
            }
        }
    }

    fn interpolate_backward(
        _x: FloatTensor<Autodiff<B, C>, 4>,
        _grad: FloatTensor<Autodiff<B, C>, 4>,
        _output_size: [usize; 2],
        _options: InterpolateOptions,
    ) -> <Autodiff<B> as Backend>::FloatTensorPrimitive<4> {
        panic!("Can't differentiate interpolate backward.");
    }
}

#[derive(Debug)]
struct MaxPool1D;

impl<B: Backend> Backward<B, 3, 1> for MaxPool1D {
    type State = (NodeID, IntTensor<B, 3>, usize, usize, usize, usize);

    fn backward(
        self,
        ops: Ops<Self::State, 1>,
        grads: &mut Gradients,
        checkpointer: &mut Checkpointer,
    ) {
        let [node_parent] = ops.parents;
        let grad = grads.consume::<B, 3>(&ops.node);
        let (x_state, indices, kernel_size, stride, padding, dilation) = ops.state;
        let x = checkpointer.retrieve_node_output(x_state);

        if let Some(node) = node_parent {
            let grad = B::max_pool1d_with_indices_backward(
                x,
                kernel_size,
                stride,
                padding,
                dilation,
                grad,
                indices,
            );

            grads.register::<B, 3>(node.id, grad.x_grad);
        }
    }
}

#[derive(Debug)]
struct MaxPool2D;

impl<B: Backend> Backward<B, 4, 1> for MaxPool2D {
    type State = (
        NodeID,
        IntTensor<B, 4>,
        [usize; 2],
        [usize; 2],
        [usize; 2],
        [usize; 2],
    );

    fn backward(
        self,
        ops: Ops<Self::State, 1>,
        grads: &mut Gradients,
        checkpointer: &mut Checkpointer,
    ) {
        let [node_parent] = ops.parents;
        let grad = grads.consume::<B, 4>(&ops.node);
        let (x_state, indices, kernel_size, stride, padding, dilation) = ops.state;
        let x = checkpointer.retrieve_node_output(x_state);

        if let Some(node) = node_parent {
            let grad = B::max_pool2d_with_indices_backward(
                x,
                kernel_size,
                stride,
                padding,
                dilation,
                grad,
                indices,
            );

            grads.register::<B, 4>(node.id, grad.x_grad);
        }
    }
}
