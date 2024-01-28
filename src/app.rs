pub(super) mod state;

use futures::{FutureExt, SinkExt, StreamExt};

use crate::repl::{
    driver::{ReplCommand, ReplEvent},
    example::ReplExample,
};

use self::state::State;

pub(crate) struct Inputs {
    pub(crate) repl_examples: Vec<ReplExample>,
    pub(crate) repl_events: futures::stream::LocalBoxStream<'static, ReplEvent>,
}

pub(crate) struct Outputs {
    pub(crate) execution_handle: futures::future::LocalBoxFuture<'static, ()>,
    pub(crate) repl_commands: futures::stream::LocalBoxStream<'static, ReplCommand>,
    pub(crate) done: futures::future::LocalBoxFuture<'static, anyhow::Result<()>>,
    pub(crate) eprintln_strings: futures::stream::LocalBoxStream<'static, String>,
}

#[derive(Debug)]
enum OutputEvent {
    Done(anyhow::Result<()>),
    ReplCommand(ReplCommand),
    Eprintln(String),
}

#[derive(Debug)]
enum InputEvent {
    ReplExample(ReplExample),
    ReplEvent(ReplEvent),
}

pub(crate) fn app(inputs: Inputs) -> Outputs {
    let Inputs {
        repl_examples,
        repl_events,
    } = inputs;

    let repl_examples = futures::stream::iter(repl_examples).map(InputEvent::ReplExample);
    let repl_events = repl_events.map(InputEvent::ReplEvent);

    let input_events =
        futures::stream::select_all([repl_examples.boxed_local(), repl_events.boxed_local()]);

    let output_events = input_events
        .scan(State::default(), |state, event| {
            let output = match event {
                InputEvent::ReplExample(repl_example) => state.repl_example(repl_example),
                InputEvent::ReplEvent(repl_event) => state.repl_event(repl_event),
            };

            let output = match output {
                Ok(output) => output,
                Err(error) => vec![OutputEvent::Done(Err(error))],
            };

            futures::future::ready(Some(output))
        })
        .flat_map(futures::stream::iter);

    let (eprintln_sender, eprintln_strings) = futures::channel::mpsc::unbounded::<String>();
    let (repl_commands_sender, repl_commands) = futures::channel::mpsc::unbounded::<ReplCommand>();
    let (done_sender, done) = futures::channel::mpsc::unbounded::<anyhow::Result<()>>();

    let execution_handle = output_events.for_each(move |output_event| match output_event {
        OutputEvent::Done(done) => {
            let mut sender = done_sender.clone();
            async move {
                sender.send(done).await.unwrap();
            }
            .boxed_local()
        }
        OutputEvent::ReplCommand(repl_command) => {
            let mut sender = repl_commands_sender.clone();
            async move {
                sender.send(repl_command).await.unwrap();
            }
            .boxed_local()
        }
        OutputEvent::Eprintln(string) => {
            let mut sender = eprintln_sender.clone();
            async move {
                sender.send(string).await.unwrap();
            }
            .boxed_local()
        }
    });

    Outputs {
        eprintln_strings: eprintln_strings.boxed_local(),
        repl_commands: repl_commands.boxed_local(),
        done: done
            .into_future()
            .map(|(next_item, _tail)| next_item.unwrap())
            .boxed_local(),
        execution_handle: execution_handle.boxed_local(),
    }
}
