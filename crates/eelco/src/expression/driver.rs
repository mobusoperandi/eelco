use crate::example_id::ExampleId;

use super::ExpressionExample;

#[derive(Debug)]
pub(crate) struct EvaluateExpression(pub(crate) ExpressionExample);

#[derive(Debug)]
pub(crate) enum ExpressionResult {
    Success(ExampleId),
    SuccessWithNonNull(ExampleId, String),
    Failure(ExampleId),
}

pub(crate) struct ExpressionDriver {
    sender: futures::channel::mpsc::UnboundedSender<ExpressionResult>,
    nix_path: camino::Utf8PathBuf,
}
impl ExpressionDriver {
    pub(crate) fn new(
        nix_path: camino::Utf8PathBuf,
    ) -> (
        Self,
        futures::stream::LocalBoxStream<'static, ExpressionResult>,
    ) {
        todo!()
    }

    pub(crate) fn init(
      mut self,
      mut commands: futures::stream::LocalBoxStream<'static, EvaluateExpression>,
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
}
