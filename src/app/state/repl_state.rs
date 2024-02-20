use crate::repl::{
    driver::{LFLine, ReplQuery},
    example::{ReplEntry, ReplExample, ReplExampleEntries},
};

#[derive(Debug)]
pub(crate) struct ReplExampleState {
    pub(crate) example: ReplExample,
    pub(crate) state: ReplSessionState,
}

impl ReplExampleState {
    pub(crate) fn new(repl_example: ReplExample) -> Self {
        Self {
            example: repl_example,
            state: Default::default(),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) enum ReplSessionState {
    #[default]
    Pending,
    Live(ReplSessionLive),
    Killing,
}

impl ReplSessionState {
    pub(crate) fn live_mut(&mut self) -> anyhow::Result<&mut ReplSessionLive> {
        let Self::Live(live) = self else {
            anyhow::bail!("session not live");
        };

        Ok(live)
    }
}

#[derive(Debug)]
pub(crate) struct ReplSessionLive {
    pub(crate) iterator: std::vec::IntoIter<ReplEntry>,
    pub(crate) expecting: ReplSessionExpecting,
}

#[derive(Debug)]
pub(crate) enum ReplSessionExpecting {
    Nothing,
    Prompt(String),
    Echo {
        acc: String,
        last_query: ReplQuery,
        expected_result: Option<ExpectedResult>,
    },
    Result {
        acc: String,
        expected_result: ExpectedResult,
    },
    BlankLine {
        saw_cr: bool,
    },
}

impl ReplSessionLive {
    pub(crate) fn new(entries: ReplExampleEntries) -> Self {
        Self {
            iterator: entries.into_iter(),
            expecting: ReplSessionExpecting::Prompt(String::new()),
        }
    }
}

impl Iterator for ReplSessionLive {
    type Item = ReplEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.iterator.next()?;
        Some(entry)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, derive_more::Deref, derive_more::Display)]
pub(crate) struct ExpectedResult(pub(crate) String);

impl From<LFLine> for ExpectedResult {
    fn from(expected_result: LFLine) -> Self {
        let expected_result = expected_result
            .as_str()
            .strip_suffix('\n')
            .unwrap()
            .to_owned();

        Self(expected_result)
    }
}
