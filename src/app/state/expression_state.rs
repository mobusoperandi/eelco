#[derive(Debug, Default)]
pub(crate) enum ExpressionExampleState {
    #[default]
    Pending,
    Spawned,
}
