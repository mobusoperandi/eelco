use futures::{FutureExt, SinkExt, StreamExt};

pub(crate) struct EprintlnDriver {
    sender: futures::channel::mpsc::UnboundedSender<Eprintlned>,
}

pub(crate) struct Eprintlned;

impl EprintlnDriver {
    pub(crate) fn new() -> (Self, futures::stream::LocalBoxStream<'static, Eprintlned>) {
        let (sender, receiver) = futures::channel::mpsc::unbounded();
        let driver = Self { sender };
        (driver, receiver.boxed_local())
    }

    pub(crate) fn init(
        mut self,
        mut lines: futures::stream::LocalBoxStream<'static, String>,
    ) -> futures::future::LocalBoxFuture<'static, ()> {
        async move {
            loop {
                let line = futures::poll!(&mut lines.next());
                if let std::task::Poll::Ready(Some(line)) = line {
                    eprintln!("{line}");
                    self.sender.send(Eprintlned).await.unwrap();
                }
                tokio::task::yield_now().await;
            }
        }
        .boxed_local()
    }
}
