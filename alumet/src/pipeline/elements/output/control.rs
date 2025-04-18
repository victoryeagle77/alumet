use anyhow::Context;
use num_enum::{FromPrimitive, IntoPrimitive};
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc, Mutex,
};
use tokio::{
    runtime,
    sync::Notify,
    task::{JoinError, JoinSet},
};

use crate::pipeline::elements::output::{run::run_async_output, AsyncOutputStream};
use crate::pipeline::matching::OutputNamePattern;
use crate::pipeline::naming::{namespace::Namespace2, OutputName};
use crate::pipeline::util::{
    channel,
    stream::{ControlledStream, SharedStreamState, StreamState},
};
use crate::pipeline::{control::matching::OutputMatcher, matching::ElementNamePattern, naming::ElementKind};
use crate::{measurement::MeasurementBuffer, pipeline::error::PipelineError};
use crate::{metrics::online::MetricReader, pipeline::naming::ElementName};

use super::{
    builder::{self, OutputBuilder},
    run::run_blocking_output,
};

/// A control messages for outputs.
#[derive(Debug)]
pub enum ControlMessage {
    Configure(ConfigureMessage),
    CreateMany(CreateManyMessage),
}

#[derive(Debug)]
pub struct ConfigureMessage {
    /// Which output(s) to reconfigure.
    pub matcher: OutputMatcher,
    /// The new state to apply to the selected output(s).
    pub new_state: TaskState,
}

#[derive(Debug)]
pub struct CreateManyMessage {
    pub builders: Vec<(OutputName, builder::SendOutputBuilder)>,
}

/// State of a (managed) output task.
#[derive(Clone, Debug, PartialEq, Eq, Copy, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum TaskState {
    Run,
    RunDiscard,
    Pause,
    StopFinish,
    #[num_enum(default)]
    StopNow,
}

pub enum SingleOutputController {
    Blocking(Arc<SharedOutputConfig>),
    Async(Arc<SharedStreamState>),
}

pub struct SharedOutputConfig {
    pub change_notifier: Notify,
    pub atomic_state: AtomicU8,
}

impl SharedOutputConfig {
    pub fn new() -> Self {
        Self {
            change_notifier: Notify::new(),
            atomic_state: AtomicU8::new(TaskState::Run as u8),
        }
    }

    pub fn set_state(&self, state: TaskState) {
        self.atomic_state.store(state as u8, Ordering::Relaxed);
        self.change_notifier.notify_one();
    }
}

impl SingleOutputController {
    pub fn set_state(&mut self, state: TaskState) {
        match self {
            SingleOutputController::Blocking(shared) => shared.set_state(state),
            SingleOutputController::Async(arc) => arc.set(StreamState::from(state as u8)),
        }
    }
}

pub(crate) struct OutputControl {
    tasks: TaskManager,
    /// Read-only access to the metrics.
    metrics: MetricReader,
}

struct TaskManager {
    spawned_tasks: JoinSet<Result<(), PipelineError>>,
    controllers: Vec<(OutputName, SingleOutputController)>,

    rx_provider: channel::ReceiverProvider,

    /// Handle of the "normal" async runtime. Used for creating new outputs.
    rt_normal: runtime::Handle,

    metrics: MetricReader,
}

impl OutputControl {
    pub fn new(rx_provider: channel::ReceiverProvider, rt_normal: runtime::Handle, metrics: MetricReader) -> Self {
        Self {
            tasks: TaskManager {
                spawned_tasks: JoinSet::new(),
                controllers: Vec::new(),
                rx_provider,
                rt_normal,
                metrics: metrics.clone(),
            },
            metrics,
        }
    }

    pub fn blocking_create_outputs(&mut self, outputs: Namespace2<OutputBuilder>) -> anyhow::Result<()> {
        let metrics = self.metrics.blocking_read();
        for ((plugin, output_name), builder) in outputs {
            let mut ctx = builder::OutputBuildContext {
                metrics: &metrics,
                metrics_r: &self.metrics.clone(),
                runtime: self.tasks.rt_normal.clone(),
            };
            let full_name = OutputName::new(plugin.clone(), output_name);
            self.tasks
                .create_output(&mut ctx, full_name, builder)
                .inspect_err(|e| log::error!("Error in output creation requested by plugin {plugin}: {e:#}"))?;
        }
        Ok(())
    }

    pub async fn create_outputs(
        &mut self,
        builders: Vec<(OutputName, builder::SendOutputBuilder)>,
    ) -> anyhow::Result<()> {
        let metrics = self.metrics.read().await;
        let mut ctx = builder::OutputBuildContext {
            metrics: &metrics,
            metrics_r: &self.metrics,
            runtime: self.tasks.rt_normal.clone(),
        };
        let n = builders.len();
        log::debug!("Creating {n} outputs...");
        let mut n_errors = 0;
        for (name, builder) in builders {
            let _ = self
                .tasks
                .create_output(&mut ctx, name.clone(), builder.into())
                .inspect_err(|e| {
                    log::error!("Error while creating source '{name}': {e:?}");
                    n_errors += 1;
                });
        }
        if n_errors == 0 {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "failed to create {n_errors}/{n} outputs (see logs above)"
            ))
        }
    }

    pub async fn handle_message(&mut self, msg: ControlMessage) -> anyhow::Result<()> {
        match msg {
            ControlMessage::Configure(msg) => self.tasks.reconfigure(msg),
            ControlMessage::CreateMany(msg) => self.create_outputs(msg.builders).await?,
        }
        Ok(())
    }

    pub async fn join_next_task(&mut self) -> Result<Result<(), PipelineError>, JoinError> {
        match self.tasks.spawned_tasks.join_next().await {
            Some(res) => res,
            None => unreachable!("join_next_task must be guarded by has_task to prevent an infinite loop"),
        }
    }

    pub fn has_task(&self) -> bool {
        !self.tasks.spawned_tasks.is_empty()
    }

    pub async fn shutdown<F>(mut self, handle_task_result: F)
    where
        F: FnMut(Result<Result<(), PipelineError>, tokio::task::JoinError>),
    {
        // Outputs naturally close when the input channel is closed,
        // but that only works when the output is running.
        // If the output is paused, it needs to be stopped with a command.
        let stop_msg = ControlMessage::Configure(ConfigureMessage {
            matcher: OutputMatcher::Name(OutputNamePattern::wildcard()),
            new_state: TaskState::StopFinish,
        });
        self.handle_message(stop_msg)
            .await
            .expect("handle_message in shutdown should not fail");

        // Close the channel and wait for all outputs to finish
        self.tasks.shutdown(handle_task_result).await;
    }

    pub fn list_elements(&self, buf: &mut Vec<ElementName>, pat: &ElementNamePattern) {
        if pat.kind == None || pat.kind == Some(ElementKind::Output) {
            buf.extend(self.tasks.controllers.iter().filter_map(|(name, _)| {
                if pat.matches(name) {
                    Some(name.to_owned().into())
                } else {
                    None
                }
            }))
        }
    }
}

impl TaskManager {
    fn create_output(
        &mut self,
        ctx: &mut builder::OutputBuildContext,
        name: OutputName,
        builder: OutputBuilder,
    ) -> anyhow::Result<()> {
        match builder {
            OutputBuilder::Blocking(builder) => self.create_blocking_output(ctx, name, builder),
            OutputBuilder::Async(builder) => self.create_async_output(ctx, name, builder),
        }
    }

    fn create_blocking_output(
        &mut self,
        ctx: &mut dyn builder::BlockingOutputBuildContext,
        name: OutputName,
        builder: Box<dyn builder::BlockingOutputBuilder>,
    ) -> anyhow::Result<()> {
        // Build the output.
        let output = builder(ctx).context("output creation failed")?;

        // Create the necessary context.
        let rx = self.rx_provider.get(); // to receive measurements
        let metrics = self.metrics.clone(); // to read metric definitions

        // Create and store the task controller.
        let config = Arc::new(SharedOutputConfig::new());
        let shared_config = config.clone();
        let control = SingleOutputController::Blocking(config);
        self.controllers.push((name.clone(), control));

        // Put the output in a Mutex to overcome the lack of tokio::spawn_scoped.
        let guarded_output = Arc::new(Mutex::new(output));

        // Spawn the task on the runtime.
        match rx {
            // Specialize on the kind of receiver at compile-time (for performance).
            channel::ReceiverEnum::Broadcast(rx) => {
                let task = run_blocking_output(name, guarded_output, rx, metrics, shared_config);
                self.spawned_tasks.spawn_on(task, &self.rt_normal);
            }
            channel::ReceiverEnum::Single(rx) => {
                let task = run_blocking_output(name, guarded_output, rx, metrics, shared_config);
                self.spawned_tasks.spawn_on(task, &self.rt_normal);
            }
        }

        Ok(())
    }

    fn create_async_output(
        &mut self,
        ctx: &mut dyn builder::AsyncOutputBuildContext,
        name: OutputName,
        builder: Box<dyn builder::AsyncOutputBuilder>,
    ) -> anyhow::Result<()> {
        use channel::MeasurementReceiver;

        fn box_controlled_stream<
            S: futures::Stream<Item = Result<MeasurementBuffer, channel::StreamRecvError>> + Send + 'static,
        >(
            stream: S,
        ) -> (AsyncOutputStream, Arc<SharedStreamState>) {
            let stream = Box::pin(ControlledStream::new(stream));
            let state = stream.state();
            (AsyncOutputStream(stream), state)
        }

        // For async outputs, we need to build the stream first
        let rx = self.rx_provider.get();
        let (stream, state) = match rx {
            channel::ReceiverEnum::Broadcast(receiver) => box_controlled_stream(receiver.into_stream()),
            channel::ReceiverEnum::Single(receiver) => box_controlled_stream(receiver.into_stream()),
        };

        // Create the output
        let output = builder(ctx, stream).context("output creation failed")?;

        // Create and store the task controller
        let control = SingleOutputController::Async(state);
        self.controllers.push((name.clone(), control));

        // Spawn the output
        let task = run_async_output(name, output);
        self.spawned_tasks.spawn_on(task, &self.rt_normal);
        Ok(())
    }

    fn reconfigure(&mut self, msg: ConfigureMessage) {
        for (name, output_config) in &mut self.controllers {
            if msg.matcher.matches(name) {
                output_config.set_state(msg.new_state);
            }
        }
    }

    async fn shutdown<F>(self, mut handle_task_result: F)
    where
        F: FnMut(Result<Result<(), PipelineError>, tokio::task::JoinError>),
    {
        // Drop the rx_provider first in order to close the channel.
        drop(self.rx_provider);
        let mut spawned_tasks = self.spawned_tasks;

        // Wait for all outputs to finish
        loop {
            match spawned_tasks.join_next().await {
                Some(res) => handle_task_result(res),
                None => break,
            }
        }
    }
}
