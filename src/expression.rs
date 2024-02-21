pub(crate) mod driver;

use crate::example_id::ExampleId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExpressionExample {
    pub(crate) id: ExampleId,
    pub(crate) expression: String,
}

impl ExpressionExample {
    pub(crate) fn new(id: ExampleId, expression: String) -> Self {
        Self { id, expression }
    }
}
