use futures::{FutureExt, SinkExt, StreamExt};
use itertools::Itertools;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::example_id::ExampleId;

#[derive(Debug, Clone, PartialEq, Eq, derive_more::Deref, derive_more::Display)]
pub(crate) struct LFLine(String);

impl std::str::FromStr for LFLine {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
        Ok(Self(s.to_string()))
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
    Spawn(pty_process::Result<ExampleId>),
    Query(ExampleId, ReplQuery, anyhow::Result<()>),
    Kill(anyhow::Result<ExampleId>),
    Read(ExampleId, std::io::Result<u8>),
}

pub(crate) struct ReplDriver {
    sessions: std::collections::BTreeMap<ExampleId, (pty_process::Pty, tokio::process::Child)>,
    sender: futures::channel::mpsc::UnboundedSender<ReplEvent>,
    nix_path: camino::Utf8PathBuf,
}

impl ReplDriver {
    pub(crate) fn new(
        nix_path: camino::Utf8PathBuf,
    ) -> (Self, futures::stream::LocalBoxStream<'static, ReplEvent>) {
        let (sender, receiver) = futures::channel::mpsc::unbounded::<ReplEvent>();
        let driver = Self {
            sessions: Default::default(),
            sender,
            nix_path,
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

                for (id, (pty, _child)) in self.sessions.iter_mut() {
                    let byte = futures::poll!(std::pin::pin!(pty.read_u8()));
                    let std::task::Poll::Ready(byte) = byte else {
                        continue;
                    };

                    match byte {
                        Ok(byte) => {
                            self.sender
                                .send(ReplEvent::Read(id.clone(), Ok(byte)))
                                .await
                                .unwrap();
                        }
                        Err(error) => {
                            self.sender
                                .send(ReplEvent::Read(id.clone(), Err(error)))
                                .await
                                .unwrap();
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
        let pty = match pty_process::Pty::new() {
            Ok(pty) => pty,
            Err(error) => {
                self.sender
                    .send(ReplEvent::Spawn(Err(error)))
                    .await
                    .unwrap();
                return;
            }
        };

        let pts = match pty.pts() {
            Ok(pts) => pts,
            Err(error) => {
                self.sender
                    .send(ReplEvent::Spawn(Err(error)))
                    .await
                    .unwrap();
                return;
            }
        };

        let child = pty_process::Command::new(&self.nix_path)
            .args(["repl", "--quiet"])
            .spawn(&pts);

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

        self.sessions.insert(id.clone(), (pty, child));
        self.sender.send(ReplEvent::Spawn(Ok(id))).await.unwrap();
    }

    async fn query(&mut self, id: ExampleId, query: ReplQuery) {
        let (pty, _child) = match self.sessions.get_mut(&id) {
            Some(pty) => pty,
            None => {
                let error = anyhow::anyhow!("no pty for {id:?}");
                self.sender
                    .send(ReplEvent::Query(id, query, Err(error)))
                    .await
                    .unwrap();
                return;
            }
        };

        let write = pty.write_all(query.as_bytes()).await;
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
