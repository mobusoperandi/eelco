use crate::{examples::Example, example_id::ExampleId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExpressionExample {

}

impl ExpressionExample {
    pub(crate) fn new(path: camino::Utf8PathBuf, line: usize, expression: String) -> Self {
        let id = ExampleId::new(path, line);
        Self { id, expression }
    }
}
