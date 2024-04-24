use burn_common::benchmark::Benchmark;

use crate::channel::ComputeChannel;
use crate::client::ComputeClient;
use crate::server::ComputeServer;

use super::AutotuneOperation;
use alloc::boxed::Box;
use alloc::string::{String, ToString};

/// A benchmark that runs on server handles
#[derive(new)]
pub struct TuneBenchmark<S: ComputeServer, C> {
    operation: Box<dyn AutotuneOperation>,
    client: ComputeClient<S, C>,
}

impl<S: ComputeServer, C: ComputeChannel<S>> Benchmark for TuneBenchmark<S, C> {
    type Args = Box<dyn AutotuneOperation>;

    fn prepare(&self) -> Self::Args {
        self.operation.clone()
    }

    fn num_samples(&self) -> usize {
        10
    }

    fn execute(&self, operation: Self::Args) {
        AutotuneOperation::execute(operation);
    }

    fn name(&self) -> String {
        "autotune".to_string()
    }

    fn sync(&self) {
        self.client.sync();
    }
}
