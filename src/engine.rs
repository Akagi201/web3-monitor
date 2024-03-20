use tokio::{
    sync::broadcast::{self, Sender},
    task::JoinSet,
};
use tokio_stream::StreamExt;

use crate::{
    log::*,
    types::{Collector, Executor, Strategy},
};

/// The main engine. This struct is responsible for orchestrating the
/// data flow between collectors, strategies, and executors.
pub struct Engine<E, A> {
    /// The set of collectors that the engine will use to collect events.
    collectors: Vec<Box<dyn Collector<E>>>,

    /// The set of strategies that the engine will use to process events.
    strategies: Vec<Box<dyn Strategy<E, A>>>,

    /// The set of executors that the engine will use to execute actions.
    executors: Vec<Box<dyn Executor<A>>>,

    /// The capacity of the event channel.
    event_channel_capacity: usize,

    /// The capacity of the action channel.
    action_channel_capacity: usize,
}

impl<E, A> Engine<E, A> {
    pub fn new() -> Self {
        Self {
            collectors: vec![],
            strategies: vec![],
            executors: vec![],
            event_channel_capacity: 512,
            action_channel_capacity: 512,
        }
    }

    pub fn with_event_channel_capacity(mut self, capacity: usize) -> Self {
        self.event_channel_capacity = capacity;
        self
    }

    pub fn with_action_channel_capacity(mut self, capacity: usize) -> Self {
        self.action_channel_capacity = capacity;
        self
    }
}

impl<E, A> Default for Engine<E, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E, A> Engine<E, A>
where
    E: Send + Clone + 'static + std::fmt::Debug,
    A: Send + Clone + 'static + std::fmt::Debug,
{
    /// Adds a collector to be used by the engine.
    pub fn add_collector(&mut self, collector: Box<dyn Collector<E>>) {
        self.collectors.push(collector);
    }

    /// Adds a strategy to be used by the engine.
    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy<E, A>>) {
        self.strategies.push(strategy);
    }

    /// Adds an executor to be used by the engine.
    pub fn add_executor(&mut self, executor: Box<dyn Executor<A>>) {
        self.executors.push(executor);
    }

    /// The core run loop of the engine. This function will spawn a thread for
    /// each collector, strategy, and executor. It will then orchestrate the
    /// data flow between them.
    pub async fn run(self) -> Result<JoinSet<()>, Box<dyn std::error::Error>> {
        let (event_sender, _): (Sender<E>, _) = broadcast::channel(self.event_channel_capacity);
        let (action_sender, _): (Sender<A>, _) = broadcast::channel(self.action_channel_capacity);

        let mut set = JoinSet::new();

        // Spawn executors in separate threads.
        for executor in self.executors {
            let mut receiver = action_sender.subscribe();
            set.spawn(async move {
                info!(target: Module::ENGINE, "starting executor... ");
                loop {
                    match receiver.recv().await {
                        Ok(action) => match executor.execute(action).await {
                            Ok(_) => {}
                            Err(e) => {
                                error!(target: Module::ENGINE, "error executing action: {}", e)
                            }
                        },
                        Err(e) => error!(target: Module::ENGINE, "error receiving action: {}", e),
                    }
                }
            });
        }

        // Spawn strategies in separate threads.
        for mut strategy in self.strategies {
            let mut event_receiver = event_sender.subscribe();
            let action_sender = action_sender.clone();
            strategy.sync_state().await?;

            set.spawn(async move {
                info!(target: Module::ENGINE, "starting strategy... ");
                loop {
                    match event_receiver.recv().await {
                        Ok(event) => {
                            for action in strategy.process_event(event).await {
                                match action_sender.send(action) {
                                    Ok(_) => {}
                                    Err(e) => error!(target: Module::ENGINE, "error sending action: {}", e),
                                }
                            }
                        }
                        Err(e) => {
                            error!(target: Module::ENGINE, "error receiving event: {}, event_receiver len: {}", e, event_receiver.len());
                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        }
                    }
                }
            });
        }

        // Spawn collectors in separate threads.
        for collector in self.collectors {
            let event_sender = event_sender.clone();
            set.spawn(async move {
                info!(target: Module::ENGINE, "starting collector... ");
                let mut event_stream = collector.get_event_stream().await.unwrap();
                while let Some(event) = event_stream.next().await {
                    match event_sender.send(event) {
                        Ok(_) => {}
                        Err(e) => error!(target: Module::ENGINE, "error sending event: {}", e),
                    }
                }
            });
        }

        Ok(set)
    }
}
