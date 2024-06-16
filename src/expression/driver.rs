use futures::{FutureExt, SinkExt, StreamExt};

use crate::example_id::ExampleId;

use super::ExpressionExample;

#[derive(Debug)]
pub(crate) struct EvaluateExpression(pub(crate) ExpressionExample);

pub(crate) struct ExpressionDriver {
    sender: futures::channel::mpsc::UnboundedSender<ExpressionEvent>,
    nix_processes: Vec<(
        ExampleId,
        futures::future::LocalBoxFuture<'static, std::io::Result<std::process::Output>>,
    )>,
}

#[derive(Debug)]
pub(crate) enum ExpressionEvent {
    Spawn(std::io::Result<ExampleId>),
    Output(std::io::Result<(ExampleId, std::process::Output)>),
}

impl ExpressionDriver {
    pub(crate) fn new() -> (
        Self,
        futures::stream::LocalBoxStream<'static, ExpressionEvent>,
    ) {
        let (sender, receiver) = futures::channel::mpsc::unbounded();
        let driver = Self {
            sender,
            nix_processes: Vec::new(),
        };
        (driver, receiver.boxed_local())
    }

    pub(crate) fn init(
        mut self,
        mut commands: futures::stream::LocalBoxStream<'static, EvaluateExpression>,
    ) -> futures::future::LocalBoxFuture<'static, ()> {
        async move {
            loop {
                let command = futures::poll!(&mut commands.next());
                if let std::task::Poll::Ready(Some(EvaluateExpression(example))) = command {
                    self.spawn_nix(example).await;
                }
                let mut i = 0;
                while i < self.nix_processes.len() {
                    let index = i;
                    i += 1;
                    let (example_id, task) = &mut self.nix_processes[index];
                    let output = futures::poll!(task);

                    let std::task::Poll::Ready(output) = output else {
                        continue;
                    };

                    self.sender
                        .send(ExpressionEvent::Output(
                            output.map(|output| (example_id.clone(), output)),
                        ))
                        .await
                        .unwrap();

                    _ = self.nix_processes.remove(index);
                }
                tokio::task::yield_now().await;
            }
        }
        .boxed_local()
    }

    async fn spawn_nix(&mut self, example: ExpressionExample) {
        let task = tokio::process::Command::new(concat!(env!("NIX_BIN_DIR"), "/nix-instantiate"))
            .args(["--expr", "--eval"])
            .arg(example.expression)
            .output();

        self.nix_processes
            .push((example.id.clone(), task.boxed_local()));
        self.sender
            .send(ExpressionEvent::Spawn(Ok(example.id)))
            .await
            .unwrap();
    }
}
