use super::ExpressionExample;

#[derive(Debug)]
pub(crate) struct EvaluateExpression(pub(crate) ExpressionExample);

pub(crate) enum ExpressionEvent {
  Success,
  SuccessWithNonNull(String),

}