use std::{
    collections::HashMap,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use gloo::timers::callback::Timeout;
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard, RwLock};

use crate::{Toast, ToastHandle};

struct EventSubscriber {
    callback: Box<dyn Fn(&ToastEvent)>,
    handle: u64,
}

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

    pub fn get(&mut self, handle: ToastHandle) -> Option<&mut Toast> {
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

#[derive(Debug, Clone)]
pub struct ToasterOptions {
    timeout: Option<Duration>,
}

impl ToasterOptions {
    pub fn with_timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    pub fn with_timeout_secs(self, secs: f64) -> Self {
        self.with_timeout(Duration::from_secs_f64(secs))
    }

    pub fn without_timeout(mut self) -> Self {
        self.timeout = None;
        self
    }
}

impl Default for ToasterOptions {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(4)),
        }
    }
}

#[derive(Clone)]
pub struct Toaster {
    state: Arc<Mutex<State>>,
    observer: Rc<RwLock<Observer>>,
    options: Arc<ToasterOptions>,
}

impl Toaster {
    pub fn new(options: ToasterOptions) -> Toaster {
        Self {
            state: Arc::new(Mutex::new(State::new())),
            observer: Rc::new(RwLock::new(Observer::default())),
            options: Arc::new(options),
        }
    }

    pub fn get(&self, handle: ToastHandle) -> Option<MappedMutexGuard<'_, Toast>> {
        let state = self.state.lock();
        MutexGuard::try_map(state, |v| v.get(handle)).ok()
    }

    /// Add toast to state
    ///
    /// # Returns
    /// Handle to the toast
    pub fn add_toast(&self, toast: Toast) -> ToastHandle {
        let mut state = self.state.lock();
        let handle = ToastHandle(state.sequence);
        state.sequence += 1;

        let timeout = match toast.timeout {
            crate::ToastTimeout::None => None,
            crate::ToastTimeout::Default => self.options.timeout,
            crate::ToastTimeout::Duration(duration) => Some(duration),
        };

        if let Some(timeout) = timeout {
            Timeout::new(timeout.as_millis() as u32, {
                let this = self.clone();
                move || {
                    this.dismiss_toast(handle, DismissReason::Timeout);
                }
            })
            .forget();
        }

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
        Self::new(ToasterOptions::default())
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
