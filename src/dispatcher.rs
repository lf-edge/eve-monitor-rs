use std::ops::Deref;

use anyhow::Result;
use crossbeam::channel::{Receiver, Sender};

use crate::traits::IAction;

/// EventDispatcher is a simple wrapper around a crossbeam channel
/// to send and receive events. It must be Clone to be able to pass
/// it around to different parts of the application.
#[derive(Debug, Clone)]
pub struct EventDispatcher<T>
where
    T: Clone,
{
    tx: Sender<T>,
    rx: Receiver<T>,
}

impl<T> EventDispatcher<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        let (tx, rx) = crossbeam::channel::unbounded();
        Self { tx, rx }
    }
    pub fn send(&self, event: T) {
        self.tx.send(event).unwrap();
    }
    pub fn recv(&self) -> Result<T> {
        Ok(self.rx.recv()?)
    }
    pub fn try_recv(&self) -> Option<T> {
        self.rx.try_recv().ok()
    }
}

impl<T> Deref for EventDispatcher<T>
where
    T: Clone,
{
    type Target = Receiver<T>;
    fn deref(&self) -> &Self::Target {
        &self.rx
    }
}

impl<A> EventDispatcher<A>
where
    A: Clone + IAction,
{
    pub fn send_action(&self, action: A) {
        self.send(action);
    }
}
