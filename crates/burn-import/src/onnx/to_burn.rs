use std::{
    env,
    fs::{self, create_dir_all},
    path::{Path, PathBuf},
};

use burn::{
    nn::PReluConfig,
    record::{FullPrecisionSettings, HalfPrecisionSettings, PrecisionSettings},
    tensor::{Element, TensorData},
};
use log::warn;

use crate::{
    burn::{
        graph::BurnGraph,
        node::{
            argmax::ArgMaxNode,
            avg_pool1d::AvgPool1dNode,
            avg_pool2d::AvgPool2dNode,
            batch_norm::BatchNormNode,
            binary::BinaryNode,
            clip::ClipNode,
            concat::ConcatNode,
            constant::{ConstantNode, ConstantValue},
            constant_of_shape::ConstantOfShapeNode,
            conv1d::Conv1dNode,
            conv2d::Conv2dNode,
            conv3d::Conv3dNode,
            conv_transpose_2d::ConvTranspose2dNode,
            conv_transpose_3d::ConvTranspose3dNode,
            dropout::DropoutNode,
            expand::ExpandNode,
            gather::GatherNode,
            gather_elements::GatherElementsNode,
            global_avg_pool::GlobalAvgPoolNode,
            layer_norm::LayerNormNode,
            linear::LinearNode,
            mask_where::WhereNode,
            matmul::MatmulNode,
            max_pool1d::MaxPool1dNode,
            max_pool2d::MaxPool2dNode,
            pad::PadNode,
            prelu::PReluNode,
            random_normal::RandomNormalNode,
            random_uniform::RandomUniformNode,
            range::RangeNode,
            reshape::ReshapeNode,
            resize::ResizeNode,
            slice::SliceNode,
            squeeze::SqueezeNode,
            sum::SumNode,
            tile::TileNode,
            unary::UnaryNode,
            unsqueeze::UnsqueezeNode,
        },
        ScalarKind, ScalarType, ShapeType, TensorKind, TensorType, Type,
    },
    format_tokens,
    logger::init_log,
};

use super::op_configuration::{
    argmax_config, avg_pool1d_config, avg_pool2d_config, batch_norm_config, clip_config,
    concat_config, conv1d_config, conv2d_config, conv3d_config, conv_transpose2d_config,
    conv_transpose3d_config, dropout_config, expand_config, flatten_config, gather_config,
    hard_sigmoid_config, layer_norm_config, leaky_relu_config, linear_config, log_softmax_config,
    max_pool1d_config, max_pool2d_config, pad_config, reduce_max_config, reduce_mean_config,
    reduce_min_config, reduce_prod_config, reduce_sum_config, reshape_config, resize_config,
    shape_config, slice_config, softmax_config, squeeze_config, tile_config, transpose_config,
    unsqueeze_config,
};
use onnx_ir::{
    convert_constant_value,
    ir::{
        ArgType, Argument as OnnxArgument, Data, ElementType, Node, NodeType, OnnxGraph,
        TensorType as OnnxTensorType,
    },
    parse_onnx,
};

pub use crate::burn::graph::RecordType;
use crate::burn::node::mean::MeanNode;

/// Generate code and states from `.onnx` files and save them to the `out_dir`.
#[derive(Debug, Default)]
pub struct ModelGen {
    out_dir: Option<PathBuf>,
    /// List of onnx files to generate source code from.
    inputs: Vec<PathBuf>,
    development: bool,
    half_precision: bool,
    record_type: RecordType,
    embed_states: bool,
}

impl ModelGen {
    /// Create a new `ModelGen`.
    pub fn new() -> Self {
        init_log().ok(); // Error when init multiple times are ignored.
        Self::default()
    }

    /// Set output directory.
    pub fn out_dir(&mut self, out_dir: &str) -> &mut Self {
        self.out_dir = Some(Path::new(out_dir).into());
        self
    }

    /// Add input file.
    pub fn input(&mut self, input: &str) -> &mut Self {
        self.inputs.push(input.into());
        self
    }

    /// Set development mode.
    ///
    /// If this is set to true, the generated model will be saved as `.graph.txt` files and model
    /// states will be saved as `.json` file.
    pub fn development(&mut self, development: bool) -> &mut Self {
        self.development = development;
        self
    }

    /// Run code generation.
    ///
    /// This function is intended to be called from `build.rs` script.
    pub fn run_from_script(&self) {
        self.run(true);
    }

    /// Run code generation.
    ///
    /// This function is intended to be called from CLI.
    pub fn run_from_cli(&self) {
        self.run(false);
    }

    /// Specify parameter precision to be saved.
    ///
    /// # Arguments
    ///
    /// * `half_precision` - If true, half precision is saved. Otherwise, full precision is saved.
    pub fn half_precision(&mut self, half_precision: bool) -> &mut Self {
        self.half_precision = half_precision;
        self
    }

    /// Specify the type of the record to be saved.
    ///
    /// # Arguments
    ///
    /// * `record_type` - The type of the record to be saved.
    pub fn record_type(&mut self, record_type: RecordType) -> &mut Self {
        self.record_type = record_type;
        self
    }

    /// Specify whether to embed states in the generated code.
    ///
    /// # Arguments
    ///
    /// * `embed_states` - If true, states are embedded in the generated code. Otherwise, states are
    ///    saved as a separate file.
    pub fn embed_states(&mut self, embed_states: bool) -> &mut Self {
        self.embed_states = embed_states;
        self
    }

    /// Run code generation.
    fn run(&self, is_build_script: bool) {
        log::info!("Starting to convert ONNX to Burn");

        // prepend the out_dir to the cargo_out_dir if this is a build script
        let out_dir = if is_build_script {
            let cargo_out_dir = env::var("OUT_DIR").expect("OUT_DIR env is not set");
            let mut path = PathBuf::from(cargo_out_dir);

            // // Append the out_dir to the cargo_out_dir
            path.push(self.out_dir.clone().unwrap());
            path
        } else {
            self.out_dir.as_ref().expect("out_dir is not set").clone()
        };

        log::debug!("Output directory: {:?}", out_dir);

        create_dir_all(&out_dir).unwrap();

        for input in self.inputs.iter() {
            let file_name = input.file_stem().unwrap();
            let out_file: PathBuf = out_dir.join(file_name);

            log::info!("Converting {:?}", input);
            log::debug!("Input file name: {:?}", file_name);
            log::debug!("Output file: {:?}", out_file);

            self.generate_model(input, out_file);
        }

        log::info!("Finished converting ONNX to Burn");
    }

    /// Generate model source code and model state.
    fn generate_model(&self, input: &PathBuf, out_file: PathBuf) {
        log::info!("Generating model from {:?}", input);
        log::debug!("Development mode: {:?}", self.development);
        log::debug!("Output file: {:?}", out_file);

        let graph = parse_onnx(input.as_ref());
        let graph = ParsedOnnxGraph(graph);

        if self.development {
            // export the graph
            let debug_graph = format!("{:#?}", graph);
            let graph_file = out_file.with_extension("graph.txt");
            log::debug!("Writing debug graph file: {:?}", graph_file);
            fs::write(graph_file, debug_graph).unwrap();
        }

        let blank_space = true;
        let top_comment = Some(format!("Generated from ONNX {input:?} by burn-import"));

        let code = if self.half_precision {
            graph
                .into_burn::<HalfPrecisionSettings>()
                .with_record(out_file.clone(), self.record_type, self.embed_states)
                .with_blank_space(blank_space)
                .with_top_comment(top_comment)
                .codegen()
        } else {
            graph
                .into_burn::<FullPrecisionSettings>()
                .with_record(out_file.clone(), self.record_type, self.embed_states)
                .with_blank_space(blank_space)
                .with_top_comment(top_comment)
                .codegen()
        };

        let code_str = format_tokens(code);
        fs::write(out_file.with_extension("rs"), code_str).unwrap();

        log::info!("Model generated");
    }
}
#[derive(Debug)]
struct ParsedOnnxGraph(OnnxGraph);
impl ParsedOnnxGraph {
    /// Converts ONNX graph to Burn graph.
    pub fn into_burn<PS: PrecisionSettings + 'static>(self) -> BurnGraph<PS> {
        let mut graph = BurnGraph::<PS>::default();

        let mut unsupported_ops = vec![];

        for node in self.0.nodes {
            match node.node_type {
                NodeType::Add => graph.register(Self::add_conversion(node)),
                NodeType::ArgMax => graph.register(Self::argmax_conversion(node)),
                NodeType::Sub => graph.register(Self::sub_conversion(node)),
                NodeType::Mul => graph.register(Self::mul_conversion(node)),
                NodeType::Div => graph.register(Self::div_conversion(node)),
                NodeType::Equal => graph.register(Self::equal_conversion(node)),
                NodeType::Erf => graph.register(Self::erf_conversion(node)),
                NodeType::Exp => graph.register(Self::exp_conversion(node)),
                NodeType::Expand => graph.register(Self::expand_conversion(node)),
                NodeType::Clip => graph.register(Self::clip_conversion(node)),
                NodeType::Cos => graph.register(Self::cos_conversion(node)),
                NodeType::Conv1d => graph.register(Self::conv1d_conversion::<PS>(node)),
                NodeType::Conv2d => graph.register(Self::conv2d_conversion::<PS>(node)),
                NodeType::Conv3d => graph.register(Self::conv3d_conversion::<PS>(node)),
                NodeType::Max => graph.register(Self::max_conversion(node)),
                NodeType::MaxPool1d => graph.register(Self::max_pool1d_conversion(node)),
                NodeType::MaxPool2d => graph.register(Self::max_pool2d_conversion(node)),
                NodeType::Mean => graph.register(Self::mean_conversion(node)),
                NodeType::PRelu => graph.register(Self::prelu_conversion::<PS>(node)),
                NodeType::AveragePool1d => graph.register(Self::avg_pool_1d_conversion(node)),
                NodeType::AveragePool2d => graph.register(Self::avg_pool_2d_conversion(node)),
                NodeType::MatMul => graph.register(Self::matmul_conversion(node)),
                NodeType::Neg => graph.register(Self::neg_conversion(node)),
                NodeType::Not => graph.register(Self::not_conversion(node)),
                NodeType::Greater => graph.register(Self::greater_conversion(node)),
                NodeType::GreaterOrEqual => graph.register(Self::greater_or_equal_conversion(node)),
                NodeType::Less => graph.register(Self::less_conversion(node)),
                NodeType::LessOrEqual => graph.register(Self::less_or_equal_conversion(node)),
                NodeType::LayerNormalization => {
                    graph.register(Self::layer_norm_conversion::<PS>(node))
                }
                NodeType::Linear => graph.register(Self::linear_conversion::<PS>(node)),
                NodeType::BatchNormalization => {
                    graph.register(Self::batch_norm_conversion::<PS>(node))
                }
                NodeType::Relu => graph.register(Self::relu_conversion(node)),
                NodeType::Gelu => graph.register(Self::gelu_conversion(node)),
                NodeType::Flatten => graph.register(Self::flatten_conversion(node)),
                NodeType::Gather => graph.register(Self::gather_conversion(node)),
                NodeType::GatherElements => graph.register(Self::gather_elements_conversion(node)),
                NodeType::HardSigmoid => graph.register(Self::hard_sigmoid_conversion(node)),
                NodeType::Log => graph.register(Self::log_conversion(node)),
                NodeType::LeakyRelu => graph.register(Self::leaky_relu_conversion(node)),
                NodeType::LogSoftmax => graph.register(Self::log_softmax_conversion(node)),
                NodeType::Softmax => graph.register(Self::softmax_conversion(node)),
                NodeType::Sqrt => graph.register(Self::sqrt_conversion(node)),
                NodeType::Tanh => graph.register(Self::tanh_conversion(node)),
                NodeType::Constant => graph.register(Self::constant_conversion::<PS>(node)),
                NodeType::Min => graph.register(Self::min_conversion(node)),
                NodeType::Range => graph.register(Self::range_conversion(node)),
                NodeType::ReduceMax => graph.register(Self::reduce_max_conversion(node)),
                NodeType::ReduceMin => graph.register(Self::reduce_min_conversion(node)),
                NodeType::ReduceMean => graph.register(Self::reduce_mean_conversion(node)),
                NodeType::ReduceProd => graph.register(Self::reduce_prod_conversion(node)),
                NodeType::ReduceSum => graph.register(Self::reduce_sum_conversion(node)),
                NodeType::Reshape => graph.register(Self::reshape_conversion(node)),
                NodeType::Resize => graph.register(Self::resize_conversion(node)),
                NodeType::Reciprocal => graph.register(Self::reciprocal_conversion(node)),
                NodeType::Shape => graph.register(Self::shape_conversion(node)),
                NodeType::Sigmoid => graph.register(Self::sigmoid_conversion(node)),
                NodeType::Sin => graph.register(Self::sin_conversion(node)),
                NodeType::Slice => graph.register(Self::slice_conversion(node)),
                NodeType::Sum => graph.register(Self::sum_conversion(node)),
                NodeType::Transpose => graph.register(Self::transpose_conversion(node)),
                NodeType::Concat => graph.register(Self::concat_conversion(node)),
                NodeType::Cast => graph.register(Self::cast_conversion(node)),
                NodeType::Dropout => graph.register(Self::dropout_conversion(node)),
                NodeType::GlobalAveragePool => {
                    graph.register(Self::global_avg_pool_conversion(node))
                }
                NodeType::ConvTranspose2d => {
                    graph.register(Self::conv_transpose2d_conversion::<PS>(node))
                }
                NodeType::ConvTranspose3d => {
                    graph.register(Self::conv_transpose3d_conversion::<PS>(node))
                }
                NodeType::Pad => graph.register(Self::pad_conversion(node)),
                NodeType::Pow => graph.register(Self::pow_conversion(node)),
                NodeType::Unsqueeze => graph.register(Self::unsqueeze_conversion(node)),
                NodeType::Where => graph.register(Self::where_conversion(node)),
                NodeType::Sign => graph.register(Self::sign_conversion(node)),
                NodeType::Squeeze => graph.register(Self::squeeze_conversion(node)),
                NodeType::RandomUniform => graph.register(Self::random_uniform_conversion(node)),
                NodeType::Tile => graph.register(Self::tile_conversion(node)),
                NodeType::RandomNormal => graph.register(Self::random_normal_conversion(node)),
                NodeType::ConstantOfShape => {
                    graph.register(Self::constant_of_shape_conversion(node))
                }
                node_type => unsupported_ops.push(node_type),
            }
        }

        if !unsupported_ops.is_empty() {
            panic!("Unsupported ops: {:?}", unsupported_ops);
        }

        // Get input and output names
        let input_names = self
            .0
            .inputs
            .iter()
            .map(|input| input.name.clone())
            .collect::<Vec<_>>();
        let output_names = self
            .0
            .outputs
            .iter()
            .map(|output| output.name.clone())
            .collect::<Vec<_>>();

        // Register inputs and outputs with the graph
        graph.register_input_output(input_names, output_names);

        graph
    }

    fn constant_conversion<PS: PrecisionSettings>(node: Node) -> ConstantNode {
        // Additional types needed for Constant:
        // use crate::burn::node::constant::{ConstantValue, TensorValue};

        let output = node.outputs.first().unwrap();

        let attr = convert_constant_value(&node);

        let const_value = match attr.ty {
            ArgType::Tensor(tensor) => {
                // Treat tensor with dim 0 as scalar
                if tensor.dim == 0 {
                    panic!("Constant tensor with dim 0 should have been converted to scalar.")
                } else {
                    let kind: TensorKind = tensor.elem_type.clone().into();
                    let dim = tensor.dim;
                    let name = node.name.clone();
                    let shape = tensor.shape.clone();

                    let tensor_data = match tensor.elem_type {
                        // TODO Review how double precision should be supported
                        ElementType::Float32 | ElementType::Float64 => {
                            serialize_data::<PS::FloatElem>(
                                attr.value.unwrap(),
                                tensor.shape.unwrap(),
                            )
                        }
                        ElementType::Int32 | ElementType::Int64 => serialize_data::<PS::IntElem>(
                            attr.value.unwrap(),
                            tensor.shape.unwrap(),
                        ),
                        // TODO support Bool tensor when it is supported by Burn
                        _ => panic!("Unsupported constant tensor type: {:?} ", tensor.elem_type),
                    };

                    ConstantValue::Tensor(TensorType::new(name, dim, kind, shape), tensor_data)
                }
            }
            ArgType::Scalar(elem_type) => match elem_type {
                ElementType::Float64 => ConstantValue::Float64(attr.value.unwrap().into_f64()),
                ElementType::Float32 => ConstantValue::Float32(attr.value.unwrap().into_f32()),
                ElementType::Int32 => ConstantValue::Int32(attr.value.unwrap().into_i32()),
                ElementType::Int64 => ConstantValue::Int64(attr.value.unwrap().into_i64()),
                ElementType::Bool => ConstantValue::Bool(attr.value.unwrap().into_bool()),
                _ => panic!("Unsupported constant tensor type: {:?} ", elem_type),
            },
            ArgType::Shape(_) => panic!("Shape is not supported as constant value."),
        };

        ConstantNode::new(node.name.clone(), const_value, Type::from(output))
    }

    fn random_uniform_conversion(node: Node) -> RandomUniformNode {
        let output = node.outputs.first().unwrap();
        // cannot use output.to_tensor_type() here, since it drops the shape info...
        let output_type = if let Type::Tensor(t) = Type::from(output) {
            t
        } else {
            panic!("RandomUniform output type is no Tensor.");
        };

        let high = node
            .attrs
            .get("high")
            .map(|val| val.clone().into_f32() as f64)
            .unwrap_or(1.0f64);
        let low = node
            .attrs
            .get("low")
            .map(|val| val.clone().into_f32() as f64)
            .unwrap_or(0.0f64);

        if node.attrs.contains_key("seed") {
            warn!("seed attribute is not supported!");
        }

        RandomUniformNode::new(output_type, low, high)
    }

    fn random_normal_conversion(node: Node) -> RandomNormalNode {
        let output = node.outputs.first().unwrap();
        // cannot use output.to_tensor_type() here, since it drops the shape info...
        let output_type = if let Type::Tensor(t) = Type::from(output) {
            t
        } else {
            panic!("RandomNormal output type is no Tensor.");
        };

        let mean = node
            .attrs
            .get("mean")
            .map(|val| val.clone().into_f32() as f64)
            .unwrap_or(0.0f64);
        let scale = node
            .attrs
            .get("scale")
            .map(|val| val.clone().into_f32() as f64)
            .unwrap_or(1.0f64);

        if node.attrs.contains_key("seed") {
            warn!("seed attribute is not supported!");
        }

        RandomNormalNode::new(output_type, mean, scale)
    }

    pub(crate) fn constant_of_shape_conversion(node: Node) -> ConstantOfShapeNode {
        // Additional types needed for ConstantOfShape:
        use crate::burn::node::constant_of_shape::ConstantValue;

        let input = node
            .inputs
            .first()
            .expect("ConstantOfShape requires an input tensor");
        let output = node.outputs.first().unwrap();

        // The value of the output elements.Should be a one-element tensor.
        // If not specified, it defaults to a tensor of value 0 and datatype float32
        // https://github.com/onnx/onnx/blob/main/docs/Operators.md#ConstantOfShape
        let value = node
            .attrs
            .get("value")
            .and_then(|val| val.clone().into_tensor().data)
            .map(|val_data| match val_data {
                // TODO: Handle Float16
                Data::Float32s(vals) => ConstantValue::from_vec(vals),
                Data::Float64s(vals) => ConstantValue::from_vec(vals),
                Data::Int32s(vals) => ConstantValue::from_vec(vals),
                Data::Int64s(vals) => ConstantValue::from_vec(vals),
                Data::Bools(vals) => ConstantValue::from_vec(vals),
                ty => panic!("Unsupported value type {:?} for ConstantOfShape!", ty),
            })
            .unwrap_or(ConstantValue::Float32(0.0f32));

        ConstantOfShapeNode::new(Type::from(input), Type::from(output), value)
    }

    fn add_conversion(node: Node) -> BinaryNode {
        let lhs = Type::from(node.inputs.first().unwrap());
        let rhs = Type::from(node.inputs.get(1).unwrap());
        let output = Type::from(node.outputs.first().unwrap());

        BinaryNode::add(lhs, rhs, output)
    }

    fn sub_conversion(node: Node) -> BinaryNode {
        let lhs = Type::from(node.inputs.first().unwrap());
        let rhs = Type::from(node.inputs.get(1).unwrap());
        let output = Type::from(node.outputs.first().unwrap());

        BinaryNode::sub(lhs, rhs, output)
    }

    fn mul_conversion(node: Node) -> BinaryNode {
        let lhs = Type::from(node.inputs.first().unwrap());
        let rhs = Type::from(node.inputs.get(1).unwrap());
        let output = Type::from(node.outputs.first().unwrap());

        BinaryNode::mul(lhs, rhs, output)
    }

    fn div_conversion(node: Node) -> BinaryNode {
        let lhs = Type::from(node.inputs.first().unwrap());
        let rhs = Type::from(node.inputs.get(1).unwrap());
        let output = Type::from(node.outputs.first().unwrap());

        BinaryNode::div(lhs, rhs, output)
    }

    fn matmul_conversion(node: Node) -> MatmulNode {
        let lhs = TensorType::from(node.inputs.first().unwrap());
        let rhs = TensorType::from(node.inputs.get(1).unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());

        MatmulNode::new(lhs, rhs, output)
    }

    fn equal_conversion(node: Node) -> BinaryNode {
        let lhs = Type::from(node.inputs.first().unwrap());
        let rhs = Type::from(node.inputs.get(1).unwrap());
        let output = Type::from(node.outputs.first().unwrap());

        BinaryNode::equal(lhs, rhs, output)
    }

    fn max_conversion(node: Node) -> BinaryNode {
        let lhs = Type::from(node.inputs.first().unwrap());
        let rhs = Type::from(node.inputs.get(1).unwrap());
        let output = Type::from(node.outputs.first().unwrap());

        BinaryNode::max_pair(lhs, rhs, output)
    }

    fn erf_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());

        UnaryNode::erf(input, output)
    }

    fn leaky_relu_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        let alpha = leaky_relu_config(&node);

        UnaryNode::leaky_relu(input, output, alpha)
    }

    fn hard_sigmoid_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        let (alpha, beta) = hard_sigmoid_config(&node);

        UnaryNode::hard_sigmoid(input, output, alpha, beta)
    }

    fn relu_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());

        UnaryNode::relu(input, output)
    }

    fn gelu_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());

        UnaryNode::gelu(input, output)
    }

    fn log_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());

        UnaryNode::log(input, output)
    }

    fn flatten_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        let (start_dim, end_dim) = flatten_config(&node);

        UnaryNode::flatten(input, output, start_dim, end_dim)
    }

    fn gather_conversion(node: Node) -> GatherNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let index = Type::from(node.inputs.get(1).unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let dim = gather_config(&node);

        GatherNode::new(input, index, output, dim)
    }

    fn gather_elements_conversion(node: Node) -> GatherElementsNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let index = TensorType::from(node.inputs.get(1).unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let dim = gather_config(&node);

        GatherElementsNode::new(input, index, output, dim)
    }

    fn transpose_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        let perm = transpose_config(&node);

        UnaryNode::transpose(input, output, perm)
    }

    fn cast_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());

        UnaryNode::cast(input, output)
    }

    fn reshape_conversion(node: Node) -> ReshapeNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let shape = reshape_config(&node);

        ReshapeNode::new(input, output, shape)
    }

    fn resize_conversion(node: Node) -> ResizeNode {
        let name = &node.name;

        let input = TensorType::from(&node.inputs[0]);

        let output = TensorType::from(node.outputs.first().unwrap());

        let (mode, scales, sizes) = resize_config(&node);

        ResizeNode::new(name, input, output, mode, scales, sizes)
    }

    fn min_conversion(node: Node) -> BinaryNode {
        let lhs = Type::from(node.inputs.first().unwrap());
        let rhs = Type::from(node.inputs.get(1).unwrap());
        let output = Type::from(node.outputs.first().unwrap());

        BinaryNode::min_pair(lhs, rhs, output)
    }

    fn range_conversion(node: Node) -> RangeNode {
        fn convert_arg_to_scalar(arg: &OnnxArgument) -> ScalarType {
            match &arg.ty {
                ArgType::Scalar(scalar) => {
                    ScalarType::new(arg.name.clone(), ScalarKind::from(scalar))
                }
                ArgType::Tensor(tensor) => {
                    if tensor.dim != 0 {
                        panic!("Range node requires scalar inputs");
                    }
                    ScalarType::new(arg.name.clone(), ScalarKind::from(&tensor.elem_type))
                }
                _ => panic!("Range node requires scalar inputs"),
            }
        }
        let output = TensorType::from(node.outputs.first().unwrap());
        let start = convert_arg_to_scalar(node.inputs.first().unwrap());
        let end = convert_arg_to_scalar(node.inputs.get(1).unwrap());
        let step = convert_arg_to_scalar(node.inputs.get(2).unwrap());

        RangeNode::new(start, end, step, output)
    }

    fn reduce_max_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        let dim = reduce_max_config(&node);

        UnaryNode::reduce_max(input, output, dim)
    }

    fn reduce_min_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        let dim = reduce_min_config(&node);

        UnaryNode::reduce_min(input, output, dim)
    }

    fn reduce_mean_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        let dim = reduce_mean_config(&node);

        UnaryNode::reduce_mean(input, output, dim)
    }

    fn reduce_prod_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        let dim = reduce_prod_config(&node);

        UnaryNode::reduce_prod(input, output, dim)
    }

    fn reduce_sum_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        let dim = reduce_sum_config(&node);

        UnaryNode::reduce_sum(input, output, dim)
    }

    fn shape_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        let (start_dim, end_dim) = shape_config(&node);

        UnaryNode::shape(input, output, start_dim, end_dim)
    }

    fn unsqueeze_conversion(node: Node) -> UnsqueezeNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let dims = unsqueeze_config(&node);

        UnsqueezeNode::new(input, output, dims)
    }

    fn where_conversion(node: Node) -> WhereNode {
        let condition = TensorType::from(node.inputs.first().unwrap());
        let x = TensorType::from(node.inputs.get(1).unwrap());
        let y = TensorType::from(node.inputs.get(2).unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());

        WhereNode::new(condition, x, y, output)
    }

    fn clip_conversion(node: Node) -> ClipNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let (min, max) = clip_config(&node);

        ClipNode::new(input, output, min, max)
    }

    fn sigmoid_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());

        UnaryNode::sigmoid(input, output)
    }

    fn sin_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());

        UnaryNode::sin(input, output)
    }

    fn slice_conversion(node: Node) -> SliceNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let ranges = slice_config(&node);

        SliceNode::new(input, output, ranges)
    }

    fn sum_conversion(node: Node) -> SumNode {
        let inputs = node.inputs.iter().map(TensorType::from).collect();
        let output = TensorType::from(node.outputs.first().unwrap());

        SumNode::new(inputs, output)
    }

    fn reciprocal_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());

        UnaryNode::reciprocal(input, output)
    }

    fn log_softmax_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        let dim = log_softmax_config(&node);

        UnaryNode::log_softmax(input, output, dim)
    }

    fn softmax_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        let dim = softmax_config(&node);

        UnaryNode::softmax(input, output, dim)
    }

    fn sqrt_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());

        UnaryNode::sqrt(input, output)
    }

    fn tanh_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());

        UnaryNode::tanh(input, output)
    }

    fn argmax_conversion(node: Node) -> ArgMaxNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let axis = argmax_config(&node);

        ArgMaxNode::new(input, output, axis)
    }

    fn concat_conversion(node: Node) -> ConcatNode {
        let inputs = node.inputs.iter().map(TensorType::from).collect();

        let output = TensorType::from(node.outputs.first().unwrap());
        let dim = concat_config(&node);

        ConcatNode::new(inputs, output, dim)
    }

    fn linear_conversion<PS: PrecisionSettings>(node: Node) -> LinearNode {
        let name = &node.name;
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let config = linear_config(&node);

        let weight = extract_data_serialize::<PS::FloatElem>(1, &node).expect("Weight is required");

        let bias = extract_data_serialize::<PS::FloatElem>(2, &node);

        LinearNode::new(name, input, output, weight, bias, config)
    }

    fn dropout_conversion(node: Node) -> DropoutNode {
        let name = &node.name;
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let config = dropout_config(&node);

        DropoutNode::new(name, input, output, config)
    }

    fn batch_norm_conversion<PS: PrecisionSettings>(node: Node) -> BatchNormNode {
        let config = batch_norm_config(&node);
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let dim = input.dim - 2;

        let gamma = extract_data_serialize::<PS::FloatElem>(1, &node).expect("Gamma is required");
        let beta = extract_data_serialize::<PS::FloatElem>(2, &node).expect("Beta is required");
        let running_mean =
            extract_data_serialize::<PS::FloatElem>(3, &node).expect("Running mean is required");
        let running_var =
            extract_data_serialize::<PS::FloatElem>(4, &node).expect("Running var is required");

        let name = &node.name;

        BatchNormNode::new(
            dim,
            name,
            input,
            output,
            gamma,
            beta,
            running_mean,
            running_var,
            config,
        )
    }

    fn layer_norm_conversion<PS: PrecisionSettings>(node: Node) -> LayerNormNode {
        let (config, full_precision) = layer_norm_config(&node);
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());

        // Scale tensor (aka gamma)
        let gamma = extract_data_serialize::<PS::FloatElem>(1, &node).expect("Gamma is required");
        // Bias (B) optional tensor
        let beta = extract_data_serialize::<PS::FloatElem>(2, &node);

        let name = &node.name;

        LayerNormNode::new(name, input, output, gamma, beta, config, full_precision)
    }

    fn conv1d_conversion<PS: PrecisionSettings>(node: Node) -> Conv1dNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let config = conv1d_config(&node);

        let bias = node.inputs.len() == 3;
        let weight = extract_data_serialize::<PS::FloatElem>(1, &node).unwrap();
        let bias = match bias {
            true => extract_data_serialize::<PS::FloatElem>(2, &node),
            false => None,
        };

        let name = &node.name;
        Conv1dNode::new(name, input, output, weight, bias, config)
    }

    fn conv2d_conversion<PS: PrecisionSettings>(node: Node) -> Conv2dNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let config = conv2d_config(&node);

        let bias = node.inputs.len() == 3;
        let weight = extract_data_serialize::<PS::FloatElem>(1, &node).unwrap();
        let bias = match bias {
            true => extract_data_serialize::<PS::FloatElem>(2, &node),
            false => None,
        };

        let name = &node.name;
        Conv2dNode::new(name, input, output, weight, bias, config)
    }

    fn conv3d_conversion<PS: PrecisionSettings>(node: Node) -> Conv3dNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let config = conv3d_config(&node);

        let bias = node.inputs.len() == 3;
        let weight = extract_data_serialize::<PS::FloatElem>(1, &node).unwrap();
        let bias = match bias {
            true => extract_data_serialize::<PS::FloatElem>(2, &node),
            false => None,
        };

        let name = &node.name;
        Conv3dNode::new(name, input, output, weight, bias, config)
    }

    fn max_pool1d_conversion(node: Node) -> MaxPool1dNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let config = max_pool1d_config(&node);

        let name = &node.name;
        MaxPool1dNode::new(name, input, output, config)
    }

    fn max_pool2d_conversion(node: Node) -> MaxPool2dNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let config = max_pool2d_config(&node);

        let name = &node.name;
        MaxPool2dNode::new(name, input, output, config)
    }

    fn mean_conversion(node: Node) -> MeanNode {
        let inputs = node.inputs.iter().map(TensorType::from).collect();
        let output = TensorType::from(node.outputs.first().unwrap());

        MeanNode::new(inputs, output)
    }

    fn prelu_conversion<PS: PrecisionSettings>(node: Node) -> PReluNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let mut weight = extract_data_serialize::<PS::FloatElem>(1, &node).unwrap();
        let config = PReluConfig::new();
        let name = &node.name;

        if weight.shape.len() > 1 {
            if weight.shape[1..].iter().product::<usize>() == 1 {
                // Burn accepts rank 1 alpha weight
                weight.shape = weight.shape[..1].to_vec();
            } else {
                panic!("Invalid PRelu weight with shape {:?}", weight.shape);
            }
        }

        PReluNode::new(name, input, output, weight, config)
    }
    fn conv_transpose2d_conversion<PS: PrecisionSettings>(node: Node) -> ConvTranspose2dNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let config = conv_transpose2d_config(&node);

        let bias = node.inputs.len() == 3;
        let weight = extract_data_serialize::<PS::FloatElem>(1, &node).unwrap();
        let bias = match bias {
            true => extract_data_serialize::<PS::FloatElem>(2, &node),
            false => None,
        };

        let name = &node.name;
        ConvTranspose2dNode::new(name, input, output, weight, bias, config)
    }
    fn conv_transpose3d_conversion<PS: PrecisionSettings>(node: Node) -> ConvTranspose3dNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let config = conv_transpose3d_config(&node);

        let bias = node.inputs.len() == 3;
        let weight = extract_data_serialize::<PS::FloatElem>(1, &node).unwrap();
        let bias = match bias {
            true => extract_data_serialize::<PS::FloatElem>(2, &node),
            false => None,
        };

        let name = &node.name;
        ConvTranspose3dNode::new(name, input, output, weight, bias, config)
    }
    fn avg_pool_1d_conversion(node: Node) -> AvgPool1dNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let config = avg_pool1d_config(&node);

        let name = &node.name;
        AvgPool1dNode::new(name, input, output, config)
    }

    fn avg_pool_2d_conversion(node: Node) -> AvgPool2dNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let config = avg_pool2d_config(&node);

        let name = &node.name;
        AvgPool2dNode::new(name, input, output, config)
    }

    fn global_avg_pool_conversion(node: Node) -> GlobalAvgPoolNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());

        let name = &node.name;

        GlobalAvgPoolNode::new(name, input, output)
    }

    fn cos_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());

        UnaryNode::cos(input, output)
    }

    fn exp_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());

        UnaryNode::exp(input, output)
    }

    fn expand_conversion(node: Node) -> ExpandNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let shape = expand_config(&node);

        ExpandNode::new(input, output, shape)
    }

    fn neg_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        UnaryNode::neg(input, output)
    }

    fn not_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        UnaryNode::not(input, output)
    }

    fn greater_conversion(node: Node) -> BinaryNode {
        let lhs = Type::from(node.inputs.first().unwrap());
        let rhs = Type::from(node.inputs.get(1).unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        BinaryNode::greater(lhs, rhs, output)
    }

    fn less_conversion(node: Node) -> BinaryNode {
        let lhs = Type::from(node.inputs.first().unwrap());
        let rhs = Type::from(node.inputs.get(1).unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        BinaryNode::lower(lhs, rhs, output)
    }

    fn greater_or_equal_conversion(node: Node) -> BinaryNode {
        let lhs = Type::from(node.inputs.first().unwrap());
        let rhs = Type::from(node.inputs.get(1).unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        BinaryNode::greater_equal(lhs, rhs, output)
    }

    fn less_or_equal_conversion(node: Node) -> BinaryNode {
        let lhs = Type::from(node.inputs.first().unwrap());
        let rhs = Type::from(node.inputs.get(1).unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        BinaryNode::lower_equal(lhs, rhs, output)
    }

    fn pad_conversion(node: Node) -> PadNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let config = pad_config(&node);

        PadNode::new(input, output, config)
    }

    fn pow_conversion(node: Node) -> BinaryNode {
        let lhs = Type::from(node.inputs.first().unwrap());
        let rhs = Type::from(node.inputs.get(1).unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        match &rhs {
            Type::Tensor(x) => match x.kind {
                TensorKind::Int => BinaryNode::powi(lhs, rhs, output),
                TensorKind::Float => BinaryNode::powf(lhs, rhs, output),
                _ => panic!("pow function requires RHS to be int or float type"),
            },
            Type::Scalar(x) => match x.kind {
                ScalarKind::Int32 | ScalarKind::Int64 => BinaryNode::powi(lhs, rhs, output),
                ScalarKind::Float32 | ScalarKind::Float64 => BinaryNode::powf(lhs, rhs, output),
                _ => panic!("pow function requires RHS to be int or float type"),
            },
            _ => panic!("pow function only supports RHS scalar or tensor types"),
        }
    }

    fn sign_conversion(node: Node) -> UnaryNode {
        let input = Type::from(node.inputs.first().unwrap());
        let output = Type::from(node.outputs.first().unwrap());
        UnaryNode::sign(input, output)
    }

    fn squeeze_conversion(node: Node) -> SqueezeNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let axes = squeeze_config(&node);

        SqueezeNode::new(input, output, axes)
    }

    fn tile_conversion(node: Node) -> TileNode {
        let input = TensorType::from(node.inputs.first().unwrap());
        let output = TensorType::from(node.outputs.first().unwrap());
        let config = tile_config(&node);

        TileNode::new(input, output, config)
    }
}

/// Extract data from node states and convert it to `TensorData`.
///
/// # Arguments
///
/// * `input_index` - The index of the input originally from input.
/// * `node` - The node where value are stored.
#[track_caller]
fn extract_data_serialize<E: Element>(input_index: usize, node: &Node) -> Option<TensorData> {
    if node.inputs.is_empty() {
        return None;
    }

    let input = node.inputs.get(input_index);
    input?;
    let input = input.unwrap();
    input.value.as_ref()?;
    let ty = input.ty.clone();

    match ty {
        ArgType::Tensor(tensor_type) => {
            let value = input.value.as_ref().expect("Value to be provided.").clone();

            Some(serialize_data::<E>(
                value.clone(),
                tensor_type.shape.unwrap().clone(),
            ))
        }
        _ => panic!("Unsupported serialization type"),
    }
}

/// Convert data to `TensorData`.
fn serialize_data<E: Element>(data: Data, shape: Vec<usize>) -> TensorData {
    match data {
        Data::Float16s(val) => TensorData::new(val, shape).convert::<E>(),
        Data::Float32s(val) => TensorData::new(val, shape).convert::<E>(),
        Data::Float64s(val) => TensorData::new(val, shape).convert::<E>(),
        Data::Int32s(val) => TensorData::new(val, shape).convert::<E>(),
        Data::Int64s(val) => TensorData::new(val, shape).convert::<E>(),
        // TODO support Bool tensor when it is supported by Burn
        _ => panic!("Unsupported tensor element type"),
    }
}

impl From<&OnnxArgument> for TensorType {
    fn from(arg: &OnnxArgument) -> Self {
        match &arg.ty {
            ArgType::Tensor(OnnxTensorType {
                elem_type: ElementType::Float16 | ElementType::Float32 | ElementType::Float64,
                dim,
                ..
            }) => TensorType::new_float(arg.name.clone(), *dim),
            ArgType::Tensor(OnnxTensorType {
                elem_type: ElementType::Int32 | ElementType::Int64,
                dim,
                ..
            }) => TensorType::new_int(arg.name.clone(), *dim),
            ArgType::Tensor(OnnxTensorType {
                elem_type: ElementType::Bool,
                dim,
                ..
            }) => TensorType::new_bool(arg.name.clone(), *dim),
            _ => panic!("Can't transform scalar to tensor."),
        }
    }
}
impl From<&OnnxArgument> for Type {
    fn from(arg: &OnnxArgument) -> Self {
        match &arg.ty {
            ArgType::Tensor(tensor) => {
                // Treat tensor with dim 0 as scalar
                if tensor.dim == 0 {
                    Type::Scalar(ScalarType::new(
                        arg.name.clone(),
                        ScalarKind::from(&tensor.elem_type),
                    ))
                } else {
                    let kind: TensorKind = tensor.elem_type.clone().into();
                    let dim = tensor.dim;
                    let name = arg.name.clone();
                    let shape = tensor.shape.clone();
                    Type::Tensor(TensorType::new(name, dim, kind, shape))
                }
            }

            ArgType::Scalar(elem_type) => {
                Type::Scalar(ScalarType::new(arg.name.clone(), elem_type.into()))
            }
            ArgType::Shape(dim) => Type::Shape(ShapeType::new(arg.name.clone(), *dim)),
        }
    }
}

impl From<&ElementType> for ScalarKind {
    fn from(elem_type: &ElementType) -> Self {
        match elem_type {
            ElementType::Float32 => ScalarKind::Float32,
            ElementType::Float64 => ScalarKind::Float64,
            ElementType::Int32 => ScalarKind::Int32,
            ElementType::Int64 => ScalarKind::Int64,
            ElementType::Bool => ScalarKind::Bool,
            ElementType::String => panic!("String tensor unsupported"),
            ElementType::Float16 => panic!("Float16 tensor unsupported"),
        }
    }
}

impl From<ElementType> for TensorKind {
    fn from(elem_type: ElementType) -> Self {
        match elem_type {
            ElementType::Float32 => TensorKind::Float,
            ElementType::Float64 => TensorKind::Float,
            ElementType::Int32 => TensorKind::Int,
            ElementType::Int64 => TensorKind::Int,
            ElementType::Bool => TensorKind::Bool,
            _ => panic!("Unsupported tensor type"),
        }
    }
}
