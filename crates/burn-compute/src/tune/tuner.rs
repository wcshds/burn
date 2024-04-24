use core::marker::PhantomData;
#[cfg(target_family = "wasm")]
use web_time::Duration;

#[cfg(not(target_family = "wasm"))]
use core::time::Duration;

use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;
use burn_common::benchmark::{Benchmark, BenchmarkComputations, BenchmarkDurations};

use crate::channel::ComputeChannel;
use crate::client::ComputeClient;
use crate::server::ComputeServer;
use crate::tune::{AutotuneOperation, AutotuneOperationSet, TuneBenchmark, TuneCache};

#[derive(Debug)]
/// Executes autotune benchmarking and caching
pub struct Tuner<S: ComputeServer, C> {
    tune_cache: TuneCache<S::AutotuneKey>,
    _channel: PhantomData<C>,
}

#[allow(clippy::new_without_default)]
impl<S: ComputeServer, C: ComputeChannel<S>> Tuner<S, C> {
    /// Returns a tuner with cache initialized from persistent cache
    pub fn new(device_id: &str) -> Self {
        Self {
            tune_cache: TuneCache::new(device_id),
            _channel: PhantomData,
        }
    }

    pub(crate) fn autotune_fastest(&self, key: &S::AutotuneKey) -> Option<usize> {
        self.tune_cache.find_fastest(key)
    }

    pub(crate) fn execute_autotune(
        &mut self,
        autotune_operation_set: Box<dyn AutotuneOperationSet<S::AutotuneKey>>,
        client: &ComputeClient<S, C>,
    ) {
        let operation = match self.tune_cache.try_cache(autotune_operation_set) {
            super::TuneCacheResult::Hit(ops) => ops,
            super::TuneCacheResult::Miss(set) => self.autotuning(set, client),
        };

        AutotuneOperation::execute(operation);
    }

    fn autotuning(
        &mut self,
        autotune_operation_set: Box<dyn AutotuneOperationSet<S::AutotuneKey>>,
        client: &ComputeClient<S, C>,
    ) -> Box<dyn AutotuneOperation> {
        let key = autotune_operation_set.key();
        let autotunables = autotune_operation_set.autotunables();
        let mut names = Vec::with_capacity(autotunables.len());

        let results: Vec<BenchmarkDurations> = autotunables
            .into_iter()
            .map(|op| {
                names.push(op.name().to_string());
                self.run_benchmark(op, client)
            })
            .collect();

        // Finds the fastest operation, stores it and returns it
        let fastest_index = self.find_fastest(results);
        let fastest_name = names.get(fastest_index).unwrap();
        log::info!("Fastest result {fastest_name}-{key}");

        self.tune_cache.cache_insert(key.clone(), fastest_index);
        #[cfg(feature = "autotune-persistent-cache")]
        {
            let checksum = autotune_operation_set.compute_checksum();
            self.tune_cache
                .persistent_cache_insert(key, checksum, fastest_index);
            self.tune_cache.save();
        }

        match self.tune_cache.try_cache(autotune_operation_set) {
            super::TuneCacheResult::Hit(ops) => ops,
            super::TuneCacheResult::Miss(_) => panic!("We just inserted, should not miss"),
        }
    }

    fn run_benchmark(
        &mut self,
        operation: Box<dyn AutotuneOperation>,
        client: &ComputeClient<S, C>,
    ) -> BenchmarkDurations {
        TuneBenchmark::new(operation, client.clone()).run()
    }

    fn find_fastest(&self, results: Vec<BenchmarkDurations>) -> usize {
        let mut smallest_duration = Duration::MAX;
        let mut fastest_tunable = None;

        for (i, result) in results.into_iter().enumerate() {
            let computed = BenchmarkComputations::new(&result);

            if computed.median < smallest_duration {
                smallest_duration = computed.median;
                fastest_tunable = Some(i);
            }
        }

        fastest_tunable.expect("At least one kernel needed. ")
    }
}
