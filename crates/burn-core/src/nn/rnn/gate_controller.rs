use crate as burn;

use crate::module::Module;
use crate::nn::Initializer;
use crate::nn::Linear;
use crate::nn::LinearConfig;
use burn_tensor::backend::Backend;

/// A GateController represents a gate in an LSTM cell. An
/// LSTM cell generally contains three gates: an input gate,
/// forget gate, and cell gate.
///
/// An Lstm gate is modeled as two linear transformations.
/// The results of these transformations are used to calculate
/// the gate's output.
#[derive(Module, Debug)]
pub struct GateController<B: Backend> {
    /// Represents the affine transformation applied to input vector
    pub(crate) input_transform: Linear<B>,
    /// Represents the affine transformation applied to the hidden state
    pub(crate) hidden_transform: Linear<B>,
}

impl<B: Backend> GateController<B> {
    /// Initialize a new [gate_controller](GateController) module.
    pub fn new(
        d_input: usize,
        d_output: usize,
        bias: bool,
        initializer: Initializer,
        device: &B::Device,
    ) -> Self {
        Self {
            input_transform: LinearConfig {
                d_input,
                d_output,
                bias,
                initializer: initializer.clone(),
            }
            .init(device),
            hidden_transform: LinearConfig {
                d_input: d_output,
                d_output,
                bias,
                initializer,
            }
            .init(device),
        }
    }

    /// Used to initialize a gate controller with known weight layers,
    /// allowing for predictable behavior. Used only for testing in
    /// lstm.
    #[cfg(test)]
    pub fn create_with_weights(
        d_input: usize,
        d_output: usize,
        bias: bool,
        initializer: Initializer,
        input_record: crate::nn::LinearRecord<B>,
        hidden_record: crate::nn::LinearRecord<B>,
    ) -> Self {
        let l1 = LinearConfig {
            d_input,
            d_output,
            bias,
            initializer: initializer.clone(),
        }
        .init(&input_record.weight.device())
        .load_record(input_record);
        let l2 = LinearConfig {
            d_input,
            d_output,
            bias,
            initializer,
        }
        .init(&hidden_record.weight.device())
        .load_record(hidden_record);

        Self {
            input_transform: l1,
            hidden_transform: l2,
        }
    }
}
