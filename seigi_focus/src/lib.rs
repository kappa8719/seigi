//! Focus management with accessibility

mod candidates;

use std::{
    rc::{Rc, Weak},
    sync::{Mutex, MutexGuard},
};

use gloo::{timers::callback::Timeout, utils::document};
use js_sys::Function;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{AddEventListenerOptions, Event, FocusEvent, HtmlElement, KeyboardEvent, MouseEvent};

macro_rules! callback {
    ($state: ident, $closure: expr) => {{
        let $state = $state.clone();
        let _closure: Closure<dyn FnMut(&Event)> = Closure::new($closure);
        Callback(_closure)
    }};
}

/// Runs given closure with acquired guard of Weak<Mutex<T>> and return the return of closure
/// Caller should ensure that weak always upgrade to rc
///
/// # Panics
/// This function assumes that Weak::upgrade and Mutex::lock does not fail, so it will panic if one
/// of these fail.
fn acquired<T, R>(weak: &Weak<Mutex<T>>, mut f: impl FnMut(MutexGuard<'_, T>) -> R) -> R {
    let rc = weak.upgrade().unwrap();
    let state = rc.lock().unwrap();
    f(state)
}

fn active_element() -> Option<HtmlElement> {
    document()
        .active_element()
        .and_then(|v| v.dyn_into::<HtmlElement>().ok())
}

/// Set immediate timeout(0ms) for focusing given element
fn schedule_focus(target: HtmlElement) {
    Timeout::new(0, move || {
        let _ = target.focus();
    })
    .forget();
}

fn target(event: &Event) -> Option<HtmlElement> {
    event
        .target()
        .and_then(|v| v.dyn_into::<HtmlElement>().ok())
}

struct Callback(Closure<dyn FnMut(&Event)>);

impl Callback {
    fn as_function(&self) -> &Function {
        self.0.as_ref().unchecked_ref()
    }
}

struct Callbacks {
    focus_in: Callback,
    pointer_down: Callback,
    click: Callback,
    key_down: Callback,
}

#[derive(Default)]
pub enum InitialFocus {
    None,
    #[default]
    Auto,
    Selector(String),
    Element(HtmlElement),
    Function(Box<dyn Fn() -> HtmlElement>),
}

#[derive(Default)]
pub struct FocusTrapHooks {
    pub activate: Option<Box<dyn Fn()>>,
    pub deactivate: Option<Box<dyn Fn()>>,
}

pub struct FocusTrapOptions {
    /// Whether trap should return focus to the last focused element before trap activation
    pub return_focus: bool,
    pub initial_focus: InitialFocus,
    /// Whether trap should deactivate when user press esc
    pub deactivate_on_escape: bool,
    /// The hooks
    pub hooks: FocusTrapHooks,
    /// The element focus trap is attached to
    pub target: HtmlElement,
}

struct State {
    options: Rc<FocusTrapOptions>,
    is_activated: bool,
    last_focus: Option<HtmlElement>,
    return_element: Option<HtmlElement>,
    callbacks: Callbacks,
}

impl State {
    fn add_listeners(&mut self) {
        let option_captures = {
            let options = AddEventListenerOptions::new();
            options.set_capture(true);
            options
        };

        let _ = document().add_event_listener_with_callback_and_add_event_listener_options(
            "focusin",
            self.callbacks.focus_in.as_function(),
            &option_captures,
        );
        let _ = document().add_event_listener_with_callback_and_add_event_listener_options(
            "mousedown",
            self.callbacks.pointer_down.as_function(),
            &option_captures,
        );
        let _ = document().add_event_listener_with_callback_and_add_event_listener_options(
            "touchstart",
            self.callbacks.pointer_down.as_function(),
            &option_captures,
        );
        let _ = document().add_event_listener_with_callback_and_add_event_listener_options(
            "click",
            self.callbacks.click.as_function(),
            &option_captures,
        );
        let _ = document().add_event_listener_with_callback_and_add_event_listener_options(
            "keydown",
            self.callbacks.key_down.as_function(),
            &option_captures,
        );
        let _ = document()
            .add_event_listener_with_callback("keydown", self.callbacks.key_down.as_function());
    }

    fn remove_listeners(&mut self) {
        let _ = document().remove_event_listener_with_callback_and_bool(
            "focusin",
            self.callbacks.focus_in.as_function(),
            true,
        );
        let _ = document().remove_event_listener_with_callback_and_bool(
            "mousedown",
            self.callbacks.pointer_down.as_function(),
            true,
        );
        let _ = document().remove_event_listener_with_callback_and_bool(
            "touchstart",
            self.callbacks.pointer_down.as_function(),
            true,
        );
        let _ = document().remove_event_listener_with_callback_and_bool(
            "click",
            self.callbacks.click.as_function(),
            true,
        );
        let _ = document().remove_event_listener_with_callback_and_bool(
            "keydown",
            self.callbacks.key_down.as_function(),
            true,
        );
        let _ = document()
            .remove_event_listener_with_callback("keydown", self.callbacks.key_down.as_function());
    }

    fn activate(&mut self) {
        if self.is_activated {
            return;
        }
        self.is_activated = true;

        self.return_element = active_element();
        self.add_listeners();
        self.initial_focus();

        if let Some(hook) = &self.options.hooks.activate {
            hook();
        }
    }

    fn deactivate(&mut self) {
        if !self.is_activated {
            return;
        }
        self.is_activated = false;

        self.remove_listeners();
        self.return_focus();

        if let Some(hook) = &self.options.hooks.deactivate {
            hook();
        }
    }

    fn initial_focus(&self) {
        let element = match &self.options.initial_focus {
            InitialFocus::None => return,
            InitialFocus::Auto => {
                match candidates::first_focus_candidate(self.options.target.unchecked_ref()) {
                    Some(element) => element,
                    None => return,
                }
            }
            InitialFocus::Selector(selector) => {
                match document().query_selector(selector).ok().flatten() {
                    Some(element) => match element.dyn_into::<HtmlElement>() {
                        Ok(element) => element,
                        Err(_) => return,
                    },
                    None => return,
                }
            }
            InitialFocus::Element(element) => element.clone(),
            InitialFocus::Function(function) => function(),
        };

        schedule_focus(element);
    }

    fn return_focus(&self) {
        if let Some(element) = &self.return_element {
            schedule_focus(element.clone());
        }
    }

    fn handle_focus_in(&mut self, event: &FocusEvent) {
        let Some(target) = target(event.unchecked_ref()) else {
            return;
        };

        if self.options.target.contains(Some(&target)) {
            self.last_focus = Some(target)
        } else {
            // the focus has escaped out of focus trap
            event.stop_immediate_propagation();

            if let Some(last_focus) = &self.last_focus {
                schedule_focus(last_focus.clone());
            }
        }
    }

    fn handle_pointer_down(&mut self, event: &Event) {
        let Some(target) = target(event) else {
            return;
        };

        if !self.options.target.contains(Some(&target)) {
            event.prevent_default();
        }
    }

    fn handle_click(&mut self, event: &MouseEvent) {
        let Some(target) = target(event.unchecked_ref()) else {
            return;
        };

        if !self.options.target.contains(Some(&target)) {
            event.prevent_default();
            event.stop_immediate_propagation();
        }
    }

    fn handle_key_down(&mut self, event: &KeyboardEvent) {
        if event.key() == "Tab" {
            let Some(target) = event.target() else {
                return;
            };
            let target = target.unchecked_ref::<HtmlElement>();
            let is_backward = event.shift_key();
            let tab_candidates = candidates::tab_candidates(self.options.target.unchecked_ref());

            if is_backward {
                let Some(first) = tab_candidates.first() else {
                    event.prevent_default();
                    return;
                };

                if target == first {
                    // If there was a first element in vec, then there must be last one too
                    let last = tab_candidates.last().unwrap();
                    schedule_focus(last.clone());
                    event.prevent_default();
                }
            } else {
                let Some(last) = tab_candidates.last() else {
                    event.prevent_default();
                    return;
                };

                if target == last {
                    let first = tab_candidates.first().unwrap();
                    schedule_focus(first.clone());
                    event.prevent_default();
                }
            }
        } else if event.key() == "Escape" && self.options.deactivate_on_escape {
            event.prevent_default();
            self.deactivate();
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        // Clean up listeners so there are no dangling listeners pointing to dropped rust closures
        self.remove_listeners();
    }
}

#[derive(Clone)]
pub struct FocusTrap {
    state: Rc<Mutex<State>>,
}

impl FocusTrap {
    /// Return true if the trap is activated
    ///
    /// This function locks the state
    pub fn is_activated(&self) -> bool {
        self.state.lock().unwrap().is_activated
    }

    /// Activates the trap
    ///
    /// Does nothing if the trap is already activated
    pub fn activate(&self) {
        self.state.lock().unwrap().activate();
    }

    /// Deactivates the trap
    ///
    /// Does nothing if the trap is already deactivated
    pub fn deactivate(&self) {
        self.state.lock().unwrap().deactivate();
    }
}

pub fn create(options: FocusTrapOptions) -> FocusTrap {
    let options = Rc::new(options);
    let state = Rc::new_cyclic(|weak: &Weak<Mutex<State>>| {
        let weak = weak.clone();
        let focus_in = callback!(weak, move |event: &Event| acquired(&weak, |mut state| {
            state.handle_focus_in(event.unchecked_ref())
        }));
        let pointer_down = callback!(weak, move |event: &Event| acquired(&weak, |mut state| {
            state.handle_pointer_down(event)
        }));
        let click = callback!(weak, move |event: &Event| acquired(&weak, |mut state| {
            state.handle_click(event.unchecked_ref())
        }));
        let key_down = callback!(weak, move |event: &Event| acquired(&weak, |mut state| {
            state.handle_key_down(event.unchecked_ref())
        }));

        Mutex::new(State {
            options,
            is_activated: false,
            last_focus: None,
            return_element: None,
            callbacks: Callbacks {
                focus_in,
                pointer_down,
                click,
                key_down,
            },
        })
    });

    FocusTrap { state }
}
