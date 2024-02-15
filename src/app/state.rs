pub(crate) mod repl_state;

use crate::{
    example_id::ExampleId,
    repl::{
        driver::{ReplCommand, ReplEvent, ReplQuery},
        example::ReplExample,
    },
};

use self::repl_state::{
    ExamplesState, ReplExampleState, ReplSessionExpecting, ReplSessionLive, ReplSessionState,
};

use super::OutputEvent;

#[derive(Default, Debug)]
pub(super) struct State {
    examples: ExamplesState,
}

impl State {
    pub(super) fn repl_example(
        &mut self,
        repl_example: ReplExample,
    ) -> anyhow::Result<Vec<OutputEvent>> {
        let id = repl_example.id.clone();

        self.examples
            .insert(id.clone(), ReplExampleState::new(repl_example))?;

        Ok(vec![OutputEvent::ReplCommand(ReplCommand::Spawn(id))])
    }

    pub(super) fn repl_event(&mut self, repl_event: ReplEvent) -> anyhow::Result<Vec<OutputEvent>> {
        match repl_event {
            ReplEvent::Spawn(spawn) => self.repl_event_spawn(spawn),
            ReplEvent::Query(id, query, result) => self.repl_event_query(id, query, result),
            ReplEvent::Kill(id) => self.repl_event_kill(id),
            ReplEvent::Read(id, result) => self.repl_event_read(id, result),
        }
    }

    fn repl_event_spawn(
        &mut self,
        spawn: Result<ExampleId, pty_process::Error>,
    ) -> anyhow::Result<Vec<OutputEvent>> {
        let id = spawn?;

        let session = self.examples.get_mut(&id)?;

        if let ReplSessionState::Live(_) = &session.state {
            return Err(anyhow::anyhow!("spawned session {session:?} already live"));
        }

        let session_live = ReplSessionLive::new(session.example.entries.clone());
        session.state = ReplSessionState::Live(session_live);
        Ok(vec![])
    }

    fn repl_event_query(
        &self,
        _id: ExampleId,
        _query: ReplQuery,
        result: anyhow::Result<()>,
    ) -> anyhow::Result<Vec<OutputEvent>> {
        result?;
        // TODO possibly store this fact
        Ok(vec![])
    }

    fn repl_event_kill(
        &mut self,
        result: anyhow::Result<ExampleId>,
    ) -> anyhow::Result<Vec<OutputEvent>> {
        let id = result?;
        self.examples.remove(&id)?;

        let events = if self.examples.is_empty() {
            vec![OutputEvent::Done(Ok(()))]
        } else {
            vec![]
        };

        Ok(events)
    }

    fn repl_event_read(
        &mut self,
        id: ExampleId,
        result: std::io::Result<u8>,
    ) -> anyhow::Result<Vec<OutputEvent>> {
        let session_live = self.examples.get_mut(&id)?;
        let session_live = session_live.state.live_mut()?;
        let ch = result?;

        let output = match &mut session_live.expecting {
            ReplSessionExpecting::Nothing => anyhow::bail!("not expecting, got {:?}", ch as char),
            ReplSessionExpecting::Prompt(acc) => {
                acc.push(ch.into());
                let string = String::from_utf8(strip_ansi_escapes::strip(acc)?)?;

                if string == "nix-repl> " {
                    session_live.expecting = ReplSessionExpecting::Nothing;
                    self.next_query(&id)?
                } else {
                    vec![]
                }
            }
            ReplSessionExpecting::Echo {
                acc,
                last_query: expected,
                expected_result,
            } => {
                acc.push(ch.into());
                if !acc.ends_with('\n') {
                    vec![]
                } else if Self::sanitize(acc)? == expected.as_str() {
                    session_live.expecting = ReplSessionExpecting::Result {
                        acc: String::new(),
                        expected_result: expected_result.clone(),
                    };
                    vec![]
                } else {
                    anyhow::bail!("actual: {acc:?}, expected: {expected:?}");
                }
            }
            ReplSessionExpecting::Result {
                acc,
                expected_result,
            } => 'arm: {
                acc.push(ch.into());

                let Some(stripped_crlf_twice) = acc.strip_suffix("\r\n\r\n") else {
                    break 'arm vec![];
                };

                let sanitized = Self::sanitize(stripped_crlf_twice)?;

                if sanitized != expected_result.as_str() {
                    anyhow::bail!(indoc::formatdoc! {"
                        {id}
                        actual (sanitized): {sanitized}
                        expected          : {expected_result}"
                    })
                }

                session_live.expecting = ReplSessionExpecting::Prompt(String::new());
                vec![]
            }
        };

        Ok(output)
    }

    fn next_query(&mut self, id: &ExampleId) -> anyhow::Result<Vec<OutputEvent>> {
        let session = self.examples.get_mut(id)?;

        let ReplSessionState::Live(session_live) = &mut session.state else {
            anyhow::bail!("expected session {id} to be live");
        };

        let Some(entry) = session_live.next() else {
            return self.session_end(id);
        };

        session_live.expecting = ReplSessionExpecting::Echo {
            acc: String::new(),
            last_query: entry.query.clone(),
            expected_result: entry.expected_result,
        };

        Ok(vec![OutputEvent::ReplCommand(ReplCommand::Query(
            id.clone(),
            entry.query.clone(),
        ))])
    }

    fn session_end(&mut self, id: &ExampleId) -> anyhow::Result<Vec<OutputEvent>> {
        let session = self.examples.get_mut(id)?;
        session.state = ReplSessionState::Killing;
        Ok(vec![
            OutputEvent::ReplCommand(ReplCommand::Kill(id.clone())),
            OutputEvent::Eprintln(Self::fmt_pass(id)),
        ])
    }

    fn fmt_pass(id: &ExampleId) -> String {
        format!("PASS: {id}")
    }

    fn sanitize(s: &str) -> anyhow::Result<String> {
        let ansi_stripped = strip_ansi_escapes::strip(s)?;
        let string = String::from_utf8(ansi_stripped)?
            .chars()
            .filter(|ch| ch != &'\r')
            .collect();
        Ok(string)
    }
}
