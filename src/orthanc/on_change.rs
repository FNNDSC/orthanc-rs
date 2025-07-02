//! The documentation for [OrthancPluginRegisterOnChangeCallback](https://orthanc.uclouvain.be/sdk/group__Callbacks.html#ga78140887a94f1afb067a15db5ee4099c)
//! recommends handling events in a separate thread. This module helps with that.
//!
//! Future work: thread pool, async runtime, ...

use crate::orthanc::bindings;
use std::sync::mpsc::{SendError, Sender};
use std::thread::JoinHandle;

/// The event-specific parameters to
/// [OrthancPluginOnChangeCallback](https://orthanc.uclouvain.be/sdk/group__Callbacks.html#gabd05790b93ac3ef7d7e91e9d8bc46295).
pub struct OnChangeEvent {
    pub change_type: bindings::OrthancPluginChangeType,
    pub resource_type: bindings::OrthancPluginResourceType,
    pub resource_id: Option<String>,
}

/// A thread and channel for handling invocations to
/// [OrthancPluginOnChangeCallback](https://orthanc.uclouvain.be/sdk/group__Callbacks.html#gabd05790b93ac3ef7d7e91e9d8bc46295).
pub struct OnChangeThread {
    handle: JoinHandle<()>,
    sender: Sender<OnChangeEvent>,
}

impl OnChangeThread {
    /// Spawn the thread.
    pub fn spawn<F: Fn(OnChangeEvent) + Send + 'static>(on_change: F) -> Self {
        let (sender, rx) = std::sync::mpsc::channel();
        let handle = std::thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                on_change(event);
            }
        });
        Self { sender, handle }
    }

    /// Send an event to the thread.
    pub fn send(&self, event: OnChangeEvent) -> Result<(), SendError<OnChangeEvent>> {
        self.sender.send(event)
    }

    /// Join this thread.
    pub fn join(self) -> std::thread::Result<()> {
        drop(self.sender);
        self.handle.join()
    }
}
