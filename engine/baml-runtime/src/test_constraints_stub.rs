//! Stub for test constraint evaluation when `cli` feature is disabled.
//! Minimal build does not pull in minijinja; constraint evaluation returns empty.

use baml_types::{BamlValue, BamlValueWithMeta, Constraint, ResponseCheck};
use indexmap::IndexMap;

use crate::internal::llm_client::LLMCompleteResponse;

/// The result of running a series of block-level constraints within a test.
#[derive(Clone, Debug, PartialEq)]
pub enum TestConstraintsResult {
    Completed {
        checks: Vec<(String, bool)>,
        failed_assert: Option<String>,
    },
    InternalError {
        details: String,
    },
}

impl TestConstraintsResult {
    pub fn empty() -> Self {
        TestConstraintsResult::Completed {
            checks: Vec::new(),
            failed_assert: None,
        }
    }
}

/// No-op: minimal build does not evaluate constraints (requires minijinja, gated behind `cli`).
pub fn evaluate_test_constraints(
    _args: &IndexMap<String, BamlValue>,
    _value: &BamlValueWithMeta<Vec<ResponseCheck>>,
    _response: &LLMCompleteResponse,
    _constraints: Vec<Constraint>,
) -> TestConstraintsResult {
    TestConstraintsResult::empty()
}
