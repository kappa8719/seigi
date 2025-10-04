use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use parking_lot::{MappedMutexGuard, Mutex, MutexGuard, RwLock};

use crate::{Toast, ToastHandle};

struct EventSubscriber {
    callback: Box<dyn Fn(&ToastEvent)>,
    handle: u64,
}

/// Actual implementation of toaster
struct State {
    toasts: HashMap<ToastHandle, Toast>,
    sequence: u32,
}

impl State {
    pub fn new() -> Self {
        Self {
            toasts: HashMap::new(),
            sequence: 0,
        }
    }

    pub fn get(&self, handle: ToastHandle) -> Option<&Toast> {
        self.toasts.get(&handle)
    }

    pub fn get_mut(&mut self, handle: ToastHandle) -> Option<&mut Toast> {
        self.toasts.get_mut(&handle)
    }
}

#[derive(Default)]
struct Observer {
    subscribers: Vec<EventSubscriber>,
}

impl Observer {
    fn subscribe(&mut self, callback: Box<dyn Fn(&ToastEvent)>) -> u64 {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let handle = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        self.subscribers.push(EventSubscriber { callback, handle });

        handle
    }

    fn unsubscribe(&mut self, handle: u64) {
        self.subscribers.retain(|v| v.handle != handle);
    }

    fn publish(&self, event: ToastEvent) {
        for subscriber in self.subscribers.iter() {
            let callback = &subscriber.callback;
            callback(&event);
        }
    }
}

#[derive(Clone)]
pub struct Toaster {
    state: Arc<Mutex<State>>,
    observer: Arc<RwLock<Observer>>,
}

impl Toaster {
    pub fn new() -> Toaster {
        Self {
            state: Arc::new(Mutex::new(State::new())),
            observer: Arc::new(RwLock::new(Observer::default())),
        }
    }

    pub fn get(&self, handle: ToastHandle) -> Option<MappedMutexGuard<'_, Toast>> {
        let state = self.state.lock();
        MutexGuard::try_map(state, |v| v.get_mut(handle)).ok()
    }

    /// Add toast to state
    ///
    /// # Returns
    /// Handle to the toast
    pub fn add_toast(&self, toast: Toast) -> ToastHandle {
        let mut state = self.state.lock();
        let handle = ToastHandle(state.sequence);
        state.sequence += 1;

        state.toasts.insert(handle, toast);
        drop(state);

        let observer = self.observer.read();
        observer.publish(ToastEvent::Create { handle });

        handle
    }

    /// Dismiss a toast of handle with given reason
    ///
    /// # Returns
    /// True if toast has been set to be dismissed, false if no toast of handle was found
    pub fn dismiss_toast(&self, handle: ToastHandle, reason: DismissReason) -> bool {
        let mut state = self.state.lock();
        let Some(toast) = state.toasts.get_mut(&handle) else {
            return false;
        };

        toast.dismiss = Some(reason.clone());
        drop(state);

        let observer = self.observer.read();
        observer.publish(ToastEvent::Dismiss { handle, reason });

        true
    }

    /// Add subscriber to state and return handle to it
    ///
    /// # Returns
    /// Handle of added subscriber
    pub fn subscribe(&self, callback: Box<dyn Fn(&ToastEvent)>) -> u64 {
        let mut observer = self.observer.write();
        observer.subscribe(callback)
    }

    /// Remove subscriber from state
    pub fn unsubscribe(&self, handle: u64) {
        let mut observer = self.observer.write();
        observer.unsubscribe(handle)
    }
}

impl Default for Toaster {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
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
#[derive(Debug, Clone)]
pub enum DismissReason {
    /// The toast has timed out
    Timeout,
    /// The user manually dismissed the toast
    User,
}
