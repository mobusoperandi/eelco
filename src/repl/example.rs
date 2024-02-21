use crate::{app::state::repl_state::ExpectedResult, example_id::ExampleId};

use super::driver::{LFLine, ReplQuery};

#[derive(Debug, Clone)]
pub(crate) struct ReplExample {
    pub(crate) id: ExampleId,
    pub(crate) entries: ReplExampleEntries,
}

impl ReplExample {
    pub(crate) fn try_new(id: ExampleId, contents: String) -> anyhow::Result<Self> {
        Ok(Self {
            id,
            entries: contents.parse()?,
        })
    }
}

#[derive(Debug, Clone, derive_more::IntoIterator)]
pub(crate) struct ReplExampleEntries(Vec<ReplEntry>);

#[derive(Debug, Default)]
struct ParseState {
    entries: Vec<ReplEntry>,
    expecting: Expecting,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum Expecting {
    #[default]
    PromptAndQuery,
    ResultOrBlankLine(ReplQuery),
    BlankLine(ReplQuery, Option<ExpectedResult>),
}

impl std::str::FromStr for ReplExampleEntries {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let final_state =
            s.split_inclusive('\n')
                .try_fold(ParseState::default(), |mut state, line| {
                    let line = LFLine::from_str(line)?;
                    match state.expecting {
                        Expecting::PromptAndQuery => {
                            let Some(line) = line.strip_prefix("nix-repl> ") else {
                                anyhow::bail!("expected prompt, found {line:?}");
                            };
                            let query = LFLine::from_str(line).unwrap();
                            let query = ReplQuery::new(query);
                            state.expecting = Expecting::ResultOrBlankLine(query);
                        }
                        Expecting::ResultOrBlankLine(query) => {
                            if line.as_str() == "\n" {
                                state.entries.push(ReplEntry::new(query, None));
                                state.expecting = Expecting::PromptAndQuery;
                            } else {
                                let expected = Some(ExpectedResult::from(line));
                                state.expecting = Expecting::BlankLine(query, expected);
                            }
                        }
                        Expecting::BlankLine(query, expected) => {
                            anyhow::ensure!(
                                line.as_str() == "\n",
                                "expected blank line, found {line:?}"
                            );
                            state.entries.push(ReplEntry::new(query, expected));
                            state.expecting = Expecting::PromptAndQuery;
                        }
                    }
                    Ok(state)
                })?;

        anyhow::ensure!(
            final_state.expecting == Expecting::PromptAndQuery,
            "failed to parse"
        );

        Ok(Self(final_state.entries))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ReplEntry {
    pub(crate) query: ReplQuery,
    pub(crate) expected_result: Option<ExpectedResult>,
}

impl ReplEntry {
    pub(crate) fn new(query: ReplQuery, expected_result: Option<ExpectedResult>) -> Self {
        Self {
            query,
            expected_result,
        }
    }
}

pub(crate) const NIX_REPL_LANG_TAG: &str = "nix-repl";
