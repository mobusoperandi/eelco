use std::{
    os::fd::{FromRawFd, IntoRawFd},
    process::Stdio,
};

use futures::{FutureExt, SinkExt, StreamExt};
use itertools::Itertools;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::example_id::ExampleId;

#[derive(Debug, Clone, PartialEq, Eq, derive_more::Deref, derive_more::Display)]
pub(crate) struct LFLine(String);

impl LFLine {
    fn validate_str(s: &str) -> anyhow::Result<()> {
        s.chars()
            .with_position()
            .try_for_each(|(position, character)| {
                use itertools::Position::*;
                match (position, character) {
                    (Last | Only, '\n') => Ok(()),
                    (Last | Only, _) => Err(anyhow::anyhow!("does not end with LF {s:?}")),
                    (_, '\r') => Err(anyhow::anyhow!("found CR {s:?}")),
                    (_, '\n') => Err(anyhow::anyhow!("newline before end {s:?}")),
                    (_, _) => Ok(()),
                }
            })?;
        Ok(())
    }
}

impl std::str::FromStr for LFLine {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::validate_str(s)?;
        Ok(Self(s.to_string()))
    }
}

impl TryFrom<String> for LFLine {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::validate_str(&s)?;
        Ok(Self(s))
    }
}

#[derive(Debug, Clone, derive_more::Deref, PartialEq, Eq)]
pub(crate) struct ReplQuery(LFLine);

impl ReplQuery {
    pub fn new(query: LFLine) -> Self {
        Self(query)
    }
}

#[derive(Debug)]
pub(crate) enum ReplCommand {
    Spawn(ExampleId),
    Query(ExampleId, ReplQuery),
    Kill(ExampleId),
}

#[derive(Debug)]
pub(crate) enum ReplEvent {
    Spawn(std::io::Result<ExampleId>),
    Query(ExampleId, ReplQuery, anyhow::Result<()>),
    Kill(anyhow::Result<ExampleId>),
    Read(ExampleId, u8),
    Error(std::io::Error),
}

pub(crate) struct ReplDriver {
    sessions: std::collections::BTreeMap<ExampleId, (tokio::process::Child, tokio::fs::File)>,
    sender: futures::channel::mpsc::UnboundedSender<ReplEvent>,
}

impl ReplDriver {
    pub(crate) fn new() -> (Self, futures::stream::LocalBoxStream<'static, ReplEvent>) {
        let (sender, receiver) = futures::channel::mpsc::unbounded::<ReplEvent>();
        let driver = Self {
            sessions: Default::default(),
            sender,
        };
        (driver, receiver.boxed_local())
    }

    pub(crate) fn init(
        mut self,
        mut commands: futures::stream::LocalBoxStream<'static, ReplCommand>,
    ) -> futures::future::LocalBoxFuture<'static, ()> {
        async move {
            loop {
                let command = futures::poll!(&mut commands.next());
                if let std::task::Poll::Ready(Some(command)) = command {
                    self.command(command).await;
                }

                for (id, (_child, child_output)) in self.sessions.iter_mut() {
                    let byte = futures::poll!(std::pin::pin!(child_output.read_u8()));
                    let std::task::Poll::Ready(byte) = byte else {
                        continue;
                    };

                    match byte {
                        Ok(byte) => {
                            self.sender
                                .send(ReplEvent::Read(id.clone(), byte))
                                .await
                                .unwrap();
                        }
                        Err(error) => {
                            self.sender.send(ReplEvent::Error(error)).await.unwrap();
                        }
                    }
                }

                tokio::task::yield_now().await;
            }
        }
        .boxed_local()
    }

    async fn command(&mut self, repl_command: ReplCommand) {
        match repl_command {
            ReplCommand::Spawn(id) => self.spawn(id).await,
            ReplCommand::Query(id, query) => self.query(id, query).await,
            ReplCommand::Kill(id) => self.kill(id).await,
        }
    }

    async fn spawn(&mut self, id: ExampleId) {
        let (read_output, write_output) = nix::unistd::pipe().unwrap();

        let child = tokio::process::Command::new(env!("NIX_CMD_PATH"))
            // even though a single `--quiet` would normally disable the pre-prompt message
            // (at the time of writing `Nix 2.21.1`), two seem to be necessary here.
            .args(["repl", "--quiet", "--quiet"])
            .stdin(Stdio::piped())
            .stdout(write_output.try_clone().unwrap())
            .stderr(write_output)
            .spawn();

        let child = match child {
            Err(error) => {
                self.sender
                    .send(ReplEvent::Spawn(Err(error)))
                    .await
                    .unwrap();
                return;
            }
            Ok(child) => child,
        };

        let read_output = unsafe { tokio::fs::File::from_raw_fd(read_output.into_raw_fd()) };
        self.sessions.insert(id.clone(), (child, read_output));
        self.sender.send(ReplEvent::Spawn(Ok(id))).await.unwrap();
    }

    async fn query(&mut self, id: ExampleId, query: ReplQuery) {
        let child = match self.sessions.get_mut(&id) {
            Some((child, _file)) => child,
            None => {
                let error = anyhow::anyhow!("no pty for {id:?}");
                self.sender
                    .send(ReplEvent::Query(id, query, Err(error)))
                    .await
                    .unwrap();
                return;
            }
        };

        let write = child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(query.as_bytes())
            .await;

        if let Err(error) = write {
            let error = anyhow::anyhow!("failed to query {error}");
            self.sender
                .send(ReplEvent::Query(id, query, Err(error)))
                .await
                .unwrap();
            return;
        }

        self.sender
            .send(ReplEvent::Query(id, query, Ok(())))
            .await
            .unwrap();
    }

    async fn kill(&mut self, id: ExampleId) {
        let Some(session) = self.sessions.remove(&id) else {
            self.sender
                .send(ReplEvent::Kill(Err(anyhow::anyhow!(
                    "no session {id:?} to kill"
                ))))
                .await
                .unwrap();
            return;
        };
        drop(session);
        self.sender.send(ReplEvent::Kill(Ok(id))).await.unwrap();
    }
}
