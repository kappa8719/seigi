use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{Toast, ToastHandle};

struct EventSubscriber {
    callback: Box<dyn Fn(&ToastEvent) + Send>,
    handle: u64,
}

pub struct ToastState {
    subscribers: Vec<EventSubscriber>,
    toasts: HashMap<ToastHandle, Toast>,
    sequence: u32,
}

impl ToastState {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
            toasts: HashMap::new(),
            sequence: 0,
        }
    }

    /// Add toast to state
    ///
    /// # Returns
    /// Handle to the toast
    pub fn add_toast(&mut self, toast: Toast) -> ToastHandle {
        let handle = ToastHandle(self.sequence);
        self.sequence += 1;

        self.toasts.insert(handle, toast);
        self.publish_event(ToastEvent::Create { handle });

        handle
    }

    /// Dismiss a toast of handle with given reason
    ///
    /// # Returns
    /// True if toast has been set to be dismissed, false if no toast of handle was found
    pub fn dismiss_toast(&mut self, handle: ToastHandle, reason: DismissReason) -> bool {
        let Some(toast) = self.toasts.get_mut(&handle) else {
            return false;
        };

        toast.dismiss = Some(reason);
        true
    }

    /// Add subscriber to state and return handle to it
    ///
    /// # Returns
    /// Handle of added subscriber
    pub fn subscribe(&mut self, callback: Box<dyn Fn(&ToastEvent) + Send>) -> u64 {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let handle = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        self.subscribers.push(EventSubscriber { callback, handle });

        handle
    }

    /// Remove subscriber from state
    pub fn unsubscribe(&mut self, handle: u64) {
        self.subscribers.retain(|v| v.handle != handle);
    }

    fn publish_event(&self, event: ToastEvent) {
        for subscriber in self.subscribers.iter() {
            let callback = &subscriber.callback;
            callback(&event);
        }
    }
}

impl Default for ToastState {
    fn default() -> Self {
        Self::new()
    }
}

pub enum ToastEvent {
    Create {
        handle: ToastHandle,
    },
    Update {
        handle: ToastHandle,
    },
    Dismiss {
        handle: ToastHandle,
        reason: DismissReason,
    },
}

/// The reason a toast is dismissed
pub enum DismissReason {
    /// The toast has timed out
    Timeout,
    /// The user manually dismissed the toast
    User,
}
