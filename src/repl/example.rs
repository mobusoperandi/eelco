use itertools::Itertools;

use crate::{app::state::repl_state::ExpectedResult, example_id::ExampleId};

use super::driver::{LFLine, ReplQuery};

#[derive(Debug, Clone)]
pub(crate) struct ReplExample {
    pub(crate) id: ExampleId,
    pub(crate) entries: ReplExampleEntries,
}

impl ReplExample {
    pub(crate) fn try_new(
        source_path: camino::Utf8PathBuf,
        line: usize,
        contents: String,
    ) -> anyhow::Result<Self> {
        let id = ExampleId::new(source_path, line);

        Ok(Self {
            id,
            entries: contents.parse()?,
        })
    }
}

#[derive(Debug, Clone, derive_more::IntoIterator)]
pub(crate) struct ReplExampleEntries(Vec<ReplEntry>);

impl std::str::FromStr for ReplExampleEntries {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let entries = s
            .split_inclusive('\n')
            .map(|line| LFLine::from_str(line).unwrap())
            .filter(|line| **line != "\n")
            .tuples::<(_, _)>()
            .map(ReplEntry::try_from)
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(Self(entries))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ReplEntry {
    pub(crate) query: ReplQuery,
    pub(crate) expected_result: ExpectedResult,
}

impl TryFrom<(LFLine, LFLine)> for ReplEntry {
    type Error = anyhow::Error;

    fn try_from((query, response): (LFLine, LFLine)) -> Result<Self, Self::Error> {
        Ok(Self {
            query: query.try_into()?,
            expected_result: ExpectedResult(response.as_str().to_owned()),
        })
    }
}

const NIX_REPL_LANG_TAG: &str = "nix-repl";

pub(crate) fn obtain(glob: &str) -> anyhow::Result<Vec<ReplExample>> {
    glob::glob(glob)?
        .map(|path| {
            let path = camino::Utf8PathBuf::try_from(path?)?;
            let contents = std::fs::read_to_string(path.clone())?;
            anyhow::Ok((path, contents))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flat_map(|(path, contents)| {
            let arena = comrak::Arena::new();
            let ast = comrak::parse_document(&arena, &contents, &comrak::ComrakOptions::default());
            ast.traverse()
                .filter_map(move |node_edge| match node_edge {
                    comrak::arena_tree::NodeEdge::Start(node) => {
                        let ast = node.data.borrow().clone();
                        Some((path.clone(), ast))
                    }
                    comrak::arena_tree::NodeEdge::End(_) => None,
                })
                .collect::<Vec<_>>()
        })
        .filter_map(|(path, ast)| {
            if let comrak::nodes::NodeValue::CodeBlock(code_block) = ast.value {
                let comrak::nodes::NodeCodeBlock { info, literal, .. } = code_block;
                if let Some(NIX_REPL_LANG_TAG) = info.split_ascii_whitespace().next() {
                    Some((path, ast.sourcepos.start.line, literal.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .map(|(path, line, contents)| ReplExample::try_new(path, line, contents))
        .try_collect()
}
