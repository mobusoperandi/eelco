pub(super) mod state;

use futures::{FutureExt, SinkExt, StreamExt};

use crate::{
    examples::Example,
    expression::driver::{EvaluateExpression, ExpressionEvent},
    repl::driver::{ReplCommand, ReplEvent},
};

use self::state::State;

pub(crate) struct Inputs {
    pub(crate) examples: Vec<Example>,
    pub(crate) repl_events: futures::stream::LocalBoxStream<'static, ReplEvent>,
    pub(crate) expression_events: futures::stream::LocalBoxStream<'static, ExpressionEvent>,
}

pub(crate) struct Outputs {
    pub(crate) execution_handle: futures::future::LocalBoxFuture<'static, ()>,
    pub(crate) repl_commands: futures::stream::LocalBoxStream<'static, ReplCommand>,
    pub(crate) expression_commands: futures::stream::LocalBoxStream<'static, EvaluateExpression>,
    pub(crate) done: futures::future::LocalBoxFuture<'static, anyhow::Result<()>>,
    pub(crate) eprintln_strings: futures::stream::LocalBoxStream<'static, String>,
}

#[derive(Debug)]
enum OutputEvent {
    Done(anyhow::Result<()>),
    ReplCommand(ReplCommand),
    ExpressionCommand(EvaluateExpression),
    Eprintln(String),
}

#[derive(Debug)]
enum InputEvent {
    Example(Example),
    ReplEvent(ReplEvent),
    ExpressionEvent(ExpressionEvent),
}

pub(crate) fn app(inputs: Inputs) -> Outputs {
    let Inputs {
        examples,
        repl_events,
        expression_events,
    } = inputs;

    let examples = futures::stream::iter(examples).map(InputEvent::Example);

    let repl_events = repl_events.map(InputEvent::ReplEvent);
    let expression_events = expression_events.map(InputEvent::ExpressionEvent);

    let input_events = futures::stream::select_all([
        examples.boxed_local(),
        repl_events.boxed_local(),
        expression_events.boxed_local(),
    ]);

    let output_events = input_events
        .scan(State::default(), |state, event| {
            let output = match event {
                InputEvent::Example(example) => state.example(example),
                InputEvent::ReplEvent(repl_event) => state.repl_event(repl_event),
                InputEvent::ExpressionEvent(expression_event) => {
                    state.expression_event(expression_event)
                }
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
    let (expression_commands_sender, expression_commands) =
        futures::channel::mpsc::unbounded::<EvaluateExpression>();
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
        OutputEvent::ExpressionCommand(evaluate_expression) => {
            let mut sender = expression_commands_sender.clone();
            async move {
                sender.send(evaluate_expression).await.unwrap();
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
        expression_commands: expression_commands.boxed_local(),
        done: done
            .into_future()
            .map(|(next_item, _tail)| next_item.unwrap())
            .boxed_local(),
        execution_handle: execution_handle.boxed_local(),
    }
}
