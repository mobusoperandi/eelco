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
            .map(|(query, expected_result)| {
                let query = ReplQuery::try_from(query)?;
                let expected_result = ExpectedResult::from(expected_result);
                Ok(ReplEntry::new(query, expected_result))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(Self(entries))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ReplEntry {
    pub(crate) query: ReplQuery,
    pub(crate) expected_result: ExpectedResult,
}

impl ReplEntry {
    pub(crate) fn new(query: ReplQuery, expected_result: ExpectedResult) -> Self {
        Self {
            query,
            expected_result,
        }
    }
}

pub(crate) const NIX_REPL_LANG_TAG: &str = "nix-repl";
