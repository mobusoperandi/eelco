#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, derive_more::Display, derive_more::Constructor)]
#[display("{}:{}", source_path, line)]
pub(crate) struct ExampleId {
    source_path: camino::Utf8PathBuf,
    line: usize,
}

impl std::fmt::Debug for ExampleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExampleId({self})")
    }
}
