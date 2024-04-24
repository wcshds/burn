use crate::stream::store::ExecutionPlanId;
use burn_tensor::repr::OperationDescription;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

/// Index used to search optimizations.
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct ExecutionPlanIndex {
    /// We can't use `HashMap<OperationDescription, Vec<ExecutionPlanId>>` since `OperationDescription`
    /// doesn't implement [`Eq`](core::cmp::Eq).
    ///
    /// `OperationDescription` can't implement `Eq` since float types don't implement it.
    ///
    /// We rely instead on [`PartialEq`](core::cmp::PartialEq) to manually handle hash collisions.
    /// This is OK because we use `relative` operations where any scalar values are set to zeros,
    /// see [`RelativeStreamConverter`](crate::stream::RelativeStreamConverter).
    mapping: HashMap<u64, Vec<(OperationDescription, usize)>>,
    starters: Vec<Vec<ExecutionPlanId>>,
}

pub enum SearchQuery<'a> {
    PlansStartingWith(&'a OperationDescription),
}

pub enum InsertQuery<'a> {
    NewPlan {
        operations: &'a [OperationDescription],
        id: ExecutionPlanId,
    },
}

impl ExecutionPlanIndex {
    /// Search optimizations with the given [query](SearchQuery).
    pub fn find(&self, query: SearchQuery<'_>) -> Vec<ExecutionPlanId> {
        match query {
            SearchQuery::PlansStartingWith(ops) => self.find_starting_with(ops),
        }
    }

    /// Register a new optimization with the given [query](InsertQuery).
    pub fn insert(&mut self, query: InsertQuery<'_>) {
        match query {
            InsertQuery::NewPlan { operations, id } => {
                if let Some(operation) = operations.first() {
                    self.insert_new_operation(operation, id)
                }
            }
        }
    }

    fn find_starting_with(&self, operation: &OperationDescription) -> Vec<ExecutionPlanId> {
        let key = self.operation_key(operation);
        let values = match self.mapping.get(&key) {
            Some(val) => val,
            None => return Vec::new(),
        };

        if values.is_empty() {
            return Vec::new();
        }

        let (_, index) = match values.iter().find(|value| &value.0 == operation) {
            Some(val) => val,
            None => return Vec::new(),
        };

        let val = match self.starters.get(*index) {
            Some(value) => value.clone(),
            None => Vec::new(),
        };

        val
    }

    fn insert_new_operation(&mut self, ops: &OperationDescription, new_id: ExecutionPlanId) {
        let key = self.operation_key(ops);
        let values = match self.mapping.get_mut(&key) {
            Some(val) => val,
            None => {
                // New starter ops.
                let index = self.starters.len();
                self.starters.push(vec![new_id]);
                self.mapping.insert(key, vec![(ops.clone(), index)]);

                return;
            }
        };
        let (_, index) = match values.iter_mut().find(|value| &value.0 == ops) {
            Some(val) => val,
            None => {
                // New with hash collision.
                let index = self.starters.len();
                self.starters.push(vec![new_id]);
                values.push((ops.clone(), index));
                return;
            }
        };

        // New optimization for an existing starter.
        self.starters
            .get_mut(*index)
            .expect("Should exist")
            .push(new_id);
    }

    // Hash the value of the first operation in a list.
    fn operation_key(&self, ops: &OperationDescription) -> u64 {
        let mut hasher = DefaultHasher::new();
        ops.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use burn_tensor::repr::{
        BinaryOperationDescription, NumericOperationDescription, ScalarOperationDescription,
        TensorDescription, TensorId, TensorStatus,
    };

    use super::*;

    #[test]
    fn should_find_optimization_id_based_on_tensor_ops() {
        let mut index = ExecutionPlanIndex::default();
        let stream_1 = [ops_1()];
        let optimization_id_1 = 0;

        index.insert(InsertQuery::NewPlan {
            operations: &stream_1,
            id: optimization_id_1,
        });

        let found = index.find(SearchQuery::PlansStartingWith(&stream_1[0]));

        assert_eq!(found, vec![optimization_id_1]);
    }

    #[test]
    fn should_support_multiple_optimization_ids_with_same_starting_ops() {
        let mut index = ExecutionPlanIndex::default();
        let stream_1 = [ops_1(), ops_2(), ops_1()];
        let stream_2 = [ops_1(), ops_1(), ops_2()];
        let optimization_id_1 = 0;
        let optimization_id_2 = 1;

        index.insert(InsertQuery::NewPlan {
            operations: &stream_1,
            id: optimization_id_1,
        });
        index.insert(InsertQuery::NewPlan {
            operations: &stream_2,
            id: optimization_id_2,
        });

        let found = index.find(SearchQuery::PlansStartingWith(&stream_1[0]));

        assert_eq!(found, vec![optimization_id_1, optimization_id_2]);
    }

    #[test]
    fn should_only_find_optimization_with_correct_starting_ops() {
        let mut index = ExecutionPlanIndex::default();
        let stream_1 = [ops_1(), ops_1()];
        let stream_2 = [ops_2(), ops_1()];
        let optimization_id_1 = 0;
        let optimization_id_2 = 1;

        index.insert(InsertQuery::NewPlan {
            operations: &stream_1,
            id: optimization_id_1,
        });
        index.insert(InsertQuery::NewPlan {
            operations: &stream_2,
            id: optimization_id_2,
        });

        let found = index.find(SearchQuery::PlansStartingWith(&stream_1[0]));

        assert_eq!(found, vec![optimization_id_1]);
    }

    #[test]
    fn should_handle_hash_collisions() {
        let mut index = ExecutionPlanIndex::default();
        let stream_1 = [ops_1(), ops_1()];
        let stream_2 = [ops_3(), ops_1()];
        let optimization_id_1 = 0;
        let optimization_id_2 = 1;

        let stream_1_key = index.operation_key(&stream_1[0]);
        let stream_2_key = index.operation_key(&stream_2[0]);

        assert_eq!(
            stream_1_key, stream_2_key,
            "Ops 1 and Ops 3 have the same hash"
        );
        assert_ne!(stream_1[0], stream_2[0], "Ops 1 and Ops 3 are different.");

        index.insert(InsertQuery::NewPlan {
            operations: &stream_1,
            id: optimization_id_1,
        });
        index.insert(InsertQuery::NewPlan {
            operations: &stream_2,
            id: optimization_id_2,
        });

        let found = index.find(SearchQuery::PlansStartingWith(&stream_1[0]));

        assert_eq!(found, vec![optimization_id_1]);
    }

    fn ops_1() -> OperationDescription {
        OperationDescription::NumericFloat(NumericOperationDescription::Add(
            BinaryOperationDescription {
                lhs: TensorDescription {
                    id: TensorId::new(0),
                    shape: vec![32, 32],
                    status: TensorStatus::ReadOnly,
                },
                rhs: TensorDescription {
                    id: TensorId::new(1),
                    shape: vec![32, 32],
                    status: TensorStatus::ReadOnly,
                },
                out: TensorDescription {
                    id: TensorId::new(2),
                    shape: vec![32, 32],
                    status: TensorStatus::NotInit,
                },
            },
        ))
    }

    fn ops_2() -> OperationDescription {
        OperationDescription::NumericFloat(NumericOperationDescription::AddScalar(
            ScalarOperationDescription {
                lhs: TensorDescription {
                    id: TensorId::new(0),
                    shape: vec![32, 32],
                    status: TensorStatus::ReadOnly,
                },
                rhs: 5.0,
                out: TensorDescription {
                    id: TensorId::new(2),
                    shape: vec![32, 32],
                    status: TensorStatus::NotInit,
                },
            },
        ))
    }

    fn ops_3() -> OperationDescription {
        OperationDescription::NumericFloat(NumericOperationDescription::Sub(
            BinaryOperationDescription {
                lhs: TensorDescription {
                    id: TensorId::new(0),
                    shape: vec![32, 32],
                    status: TensorStatus::ReadOnly,
                },
                rhs: TensorDescription {
                    id: TensorId::new(1),
                    shape: vec![32, 32],
                    status: TensorStatus::ReadOnly,
                },
                out: TensorDescription {
                    id: TensorId::new(2),
                    shape: vec![32, 32],
                    status: TensorStatus::NotInit,
                },
            },
        ))
    }
}
