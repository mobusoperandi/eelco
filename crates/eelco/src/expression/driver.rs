use crate::example_id::ExampleId;

use super::ExpressionExample;

#[derive(Debug)]
pub(crate) struct EvaluateExpression(pub(crate) ExpressionExample);

#[derive(Debug)]
pub(crate) enum ExpressionResult {
    Success(ExampleId),
    SuccessWithNonNull(ExampleId, String),
    Failure(ExampleId),
}

pub(crate) struct ExpressionDriver {
    sessions: std::collections::BTreeMap<ExampleId, (pty_process::Pty, tokio::process::Child)>,
    sender: futures::channel::mpsc::UnboundedSender<ReplEvent>,
    nix_path: camino::Utf8PathBuf,
}
