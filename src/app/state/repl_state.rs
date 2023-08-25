use crate::{
    example_id::ExampleId,
    repl::{
        driver::ReplQuery,
        example::{ReplEntry, ReplExample, ReplExampleEntries},
    },
};

#[derive(Debug, Default)]
pub(crate) struct ReplState(std::collections::BTreeMap<ExampleId, ReplSession>);

impl ReplState {
    pub(crate) fn insert(&mut self, id: ExampleId, session: ReplSession) -> anyhow::Result<()> {
        if self.0.insert(id.clone(), session).is_some() {
            anyhow::bail!("duplicate session id {id:?}");
        };
        Ok(())
    }

    pub(crate) fn get_mut(&mut self, id: &ExampleId) -> anyhow::Result<&mut ReplSession> {
        self.0
            .get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("repl session not found {id:?}"))
    }

    pub(crate) fn remove(&mut self, id: &ExampleId) -> anyhow::Result<ReplSession> {
        self.0
            .remove(id)
            .ok_or_else(|| anyhow::anyhow!("repl session not found {id:?}"))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug)]
pub(crate) struct ReplSession {
    pub(crate) example: ReplExample,
    pub(crate) state: ReplSessionState,
}

impl ReplSession {
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
        expected_result: ExpectedResult,
    },
    Result {
        acc: String,
        expected_result: ExpectedResult,
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

#[derive(Debug, Clone, derive_more::Deref, derive_more::Display)]
pub(crate) struct ExpectedResult(pub(crate) String);
