use super::{Node, NodeCodegen, SerializationBackend};
use crate::burn::{BurnImports, OtherType, Scope, TensorType, ToTokens, Type};
use burn::{
    module::{ConstantRecord, Param, ParamId},
    nn::{LayerNormConfig, LayerNormRecord},
    record::{PrecisionSettings, Record},
    tensor::{DataSerialize, Tensor},
};
use proc_macro2::TokenStream;
use quote::quote;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct LayerNormNode<PS: PrecisionSettings> {
    pub field: OtherType,
    pub input: TensorType,
    pub output: TensorType,
    pub gamma: DataSerialize<PS::FloatElem>,        // Scale
    pub beta: Option<DataSerialize<PS::FloatElem>>, // Bias (B)
    pub config: LayerNormConfig,
    pub full_precision: bool,
}

impl<PS: PrecisionSettings> LayerNormNode<PS> {
    pub fn new<S: AsRef<str>>(
        name: S,
        input: TensorType,
        output: TensorType,
        gamma: DataSerialize<PS::FloatElem>,
        beta: Option<DataSerialize<PS::FloatElem>>,
        config: LayerNormConfig,
        full_precision: bool,
    ) -> Self {
        Self {
            field: OtherType::new(
                name,
                quote! {
                    LayerNorm<B>
                },
            ),
            input,
            output,
            gamma,
            beta,
            config,
            full_precision,
        }
    }
}

impl<PS: PrecisionSettings> NodeCodegen<PS> for LayerNormNode<PS> {
    fn input_types(&self) -> Vec<Type> {
        vec![Type::Tensor(self.input.clone())]
    }
    fn output_types(&self) -> Vec<Type> {
        vec![Type::Tensor(self.output.clone())]
    }
    fn field_type(&self) -> Option<Type> {
        Some(Type::Other(self.field.clone()))
    }

    fn field_init(&self) -> Option<TokenStream> {
        let name = &self.field.name;
        let num_features = self.config.d_model.to_tokens();
        let epsilon = self.config.epsilon;

        let tokens = quote! {
            let #name = LayerNormConfig::new(#num_features)
                .with_epsilon(#epsilon)
                .init(device);
        };

        Some(tokens)
    }

    fn field_serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let device = Default::default();
        let record = LayerNormRecord::<SerializationBackend> {
            gamma: Param::initialized(
                ParamId::new(),
                Tensor::from_data(self.gamma.clone().convert(), &device),
            ),
            beta: Param::initialized(
                ParamId::new(),
                if let Some(beta) = self.beta.clone() {
                    Tensor::from_data(beta.convert(), &device)
                } else {
                    Tensor::zeros([self.config.d_model], &device)
                },
            ),
            epsilon: ConstantRecord::new(),
        };

        let item = Record::into_item::<PS>(record);
        item.serialize(serializer)
    }

    fn forward(&self, scope: &mut Scope, node_position: usize) -> TokenStream {
        let input = scope.tensor_use_owned(&self.input, node_position);
        let output = &self.output.name;
        let field = &self.field.name;

        // TODO: handle self.full_precision
        quote! {
            let #output = self.#field.forward(#input);
        }
    }
    fn register_imports(&self, imports: &mut BurnImports) {
        imports.register("burn::nn::LayerNorm");
        imports.register("burn::nn::LayerNormConfig");
    }

    fn into_node(self) -> Node<PS> {
        Node::LayerNorm(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::burn::{graph::BurnGraph, node::test::assert_tokens, TensorType};
    use burn::{record::FullPrecisionSettings, tensor::Data};

    #[test]
    fn test_codegen() {
        let mut graph = BurnGraph::<FullPrecisionSettings>::default();

        graph.register(LayerNormNode::new(
            "norm",
            TensorType::new_float("input", 4),
            TensorType::new_float("output", 4),
            Data::from([2.]).serialize(),
            Some(Data::from([2.]).serialize()),
            LayerNormConfig::new(128),
            true, // full_precision isn't taken into account
        ));

        graph.register_input_output(vec!["input".to_string()], vec!["output".to_string()]);

        let expected = quote! {
            use burn::{
                module::Module,
                tensor::{backend::Backend, Tensor},
            };
            use burn::nn::LayerNorm;
            use burn::nn::LayerNormConfig;

            #[derive(Module, Debug)]
            pub struct Model <B: Backend> {
                norm: LayerNorm<B>,
                phantom: core::marker::PhantomData<B>,
            }

            impl<B: Backend> Model <B> {
                #[allow(unused_variables)]
                pub fn new(device: &B::Device) -> Self {
                    let norm = LayerNormConfig::new(128)
                        .with_epsilon(0.00001f64)
                        .init(device);

                    Self {
                        norm,
                        phantom: core::marker::PhantomData,
                    }
                }
                #[allow(clippy::let_and_return, clippy::approx_constant)]
                pub fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 4> {
                    let output = self.norm.forward(input);

                    output
                }
            }
        };

        assert_tokens(graph.codegen(), expected);
    }
}
