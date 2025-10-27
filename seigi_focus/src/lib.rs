//! Focus management with accessibility

mod candidates;

use std::{
    rc::{Rc, Weak},
    sync::{Mutex, MutexGuard},
};

use gloo::{
    timers::callback::Timeout,
    utils::{body, document},
};
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

/// Gets document.activeElement
fn active_element() -> Option<HtmlElement> {
    document()
        .active_element()
        .and_then(|v| v.dyn_into::<HtmlElement>().ok())
}

/// Sets immediate timeout(0ms) for focusing given element
fn schedule_focus(target: HtmlElement) {
    Timeout::new(0, move || {
        let _ = target.focus();
    })
    .forget();
}

/// Gets the target of Event as HtmlElement
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

/// The method trap will use to decide the initial focus element
#[derive(Default)]
pub enum InitialFocus {
    /// The trap doesn't do initial focus
    None,
    /// The trap focuses the first focusable element inside the trap
    #[default]
    Auto,
    /// The trap focuses the first element that matches given selector inside the trap
    Selector(String),
    /// The trap focuses given element
    Element(HtmlElement),
    /// The trap focuses returned element by the function
    Function(Box<dyn Fn() -> HtmlElement>),
}

/// Hooks to [FocusTrap]
#[derive(Default)]
pub struct FocusTrapHooks {
    /// Called when the trap is activated
    pub activate: Option<Box<dyn Fn()>>,
    /// Called when the trap is deactivated
    pub deactivate: Option<Box<dyn Fn()>>,
}

/// Options of [FocusTrap]
pub struct FocusTrapOptions {
    /// Whether trap should return focus to the last focused element before trap activation
    pub return_focus: bool,
    pub initial_focus: InitialFocus,
    /// Whether trap should deactivate when user press esc
    pub deactivate_on_escape: bool,
    /// The hooks
    pub hooks: FocusTrapHooks,
    /// The scope trap is affected.
    ///
    /// Elements outside the scope are not affected by the trap
    pub scope: HtmlElement,
    /// The element focus trap is attached to
    pub target: HtmlElement,
}

impl FocusTrapOptions {
    pub fn builder() -> FocusTrapOptionsBuilder {
        FocusTrapOptionsBuilder::new()
    }
}

/// A builder struct of [FocusTrapOptions]
pub struct FocusTrapOptionsBuilder {
    return_focus: bool,
    initial_focus: InitialFocus,
    deactivate_on_escape: bool,
    hooks: FocusTrapHooks,
    scope: HtmlElement,
    target: Option<HtmlElement>,
}

impl Default for FocusTrapOptionsBuilder {
    fn default() -> Self {
        Self {
            return_focus: true,
            initial_focus: InitialFocus::default(),
            deactivate_on_escape: false,
            hooks: FocusTrapHooks::default(),
            scope: body(),
            target: None,
        }
    }
}

impl FocusTrapOptionsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn return_focus(mut self, return_focus: bool) -> Self {
        self.return_focus = return_focus;
        self
    }

    pub fn initial_focus(mut self, initial_focus: InitialFocus) -> Self {
        self.initial_focus = initial_focus;
        self
    }

    pub fn deactivate_on_escape(mut self, deactivate_on_escape: bool) -> Self {
        self.deactivate_on_escape = deactivate_on_escape;
        self
    }

    pub fn hooks(mut self, hooks: FocusTrapHooks) -> Self {
        self.hooks = hooks;
        self
    }

    pub fn scope(mut self, scope: HtmlElement) -> Self {
        self.scope = scope;
        self
    }

    pub fn target(mut self, target: HtmlElement) -> Self {
        self.target = Some(target);
        self
    }

    /// Builds into [FocusTrapOptions]
    ///
    /// # Panics
    /// This method panics if target field is not set
    pub fn build(self) -> FocusTrapOptions {
        FocusTrapOptions {
            return_focus: self.return_focus,
            initial_focus: self.initial_focus,
            deactivate_on_escape: self.deactivate_on_escape,
            hooks: self.hooks,
            scope: self.scope,
            target: self
                .target
                .expect("target must be set to build FocusTrapOptions"),
        }
    }
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

        let scope = &self.options.scope;

        let _ = scope.add_event_listener_with_callback_and_add_event_listener_options(
            "focusin",
            self.callbacks.focus_in.as_function(),
            &option_captures,
        );
        let _ = scope.add_event_listener_with_callback_and_add_event_listener_options(
            "mousedown",
            self.callbacks.pointer_down.as_function(),
            &option_captures,
        );
        let _ = scope.add_event_listener_with_callback_and_add_event_listener_options(
            "touchstart",
            self.callbacks.pointer_down.as_function(),
            &option_captures,
        );
        let _ = scope.add_event_listener_with_callback_and_add_event_listener_options(
            "click",
            self.callbacks.click.as_function(),
            &option_captures,
        );
        let _ = scope.add_event_listener_with_callback_and_add_event_listener_options(
            "keydown",
            self.callbacks.key_down.as_function(),
            &option_captures,
        );
        let _ = scope
            .add_event_listener_with_callback("keydown", self.callbacks.key_down.as_function());
    }

    fn remove_listeners(&mut self) {
        let scope = &self.options.scope;

        let _ = scope.remove_event_listener_with_callback_and_bool(
            "focusin",
            self.callbacks.focus_in.as_function(),
            true,
        );
        let _ = scope.remove_event_listener_with_callback_and_bool(
            "mousedown",
            self.callbacks.pointer_down.as_function(),
            true,
        );
        let _ = scope.remove_event_listener_with_callback_and_bool(
            "touchstart",
            self.callbacks.pointer_down.as_function(),
            true,
        );
        let _ = scope.remove_event_listener_with_callback_and_bool(
            "click",
            self.callbacks.click.as_function(),
            true,
        );
        let _ = scope.remove_event_listener_with_callback_and_bool(
            "keydown",
            self.callbacks.key_down.as_function(),
            true,
        );
        let _ = scope
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

            let body_tab_candidates = {
                let container = &self.options.target;
                let scope = &self.options.scope;
                candidates::candidates(body().unchecked_ref(), move |v| {
                    candidates::is_tabbable(v)
                        && (!scope.contains(Some(v.unchecked_ref()))
                            || container.contains(Some(v.unchecked_ref())))
                })
            };
            let container_tab_candidates =
                candidates::tab_candidates(self.options.target.unchecked_ref());

            if is_backward {
                let Some(first) = container_tab_candidates.first() else {
                    event.prevent_default();
                    return;
                };

                if target == first {
                    let position = body_tab_candidates
                        .iter()
                        .position(|v| v == target)
                        .unwrap();

                    if position == 0 {
                        // If there was a first element in vec, then there must be last one too
                        let last = body_tab_candidates.last().unwrap();
                        schedule_focus(last.clone());
                    } else {
                        let target = body_tab_candidates
                            .get(position - 1)
                            .unwrap_or_else(|| body_tab_candidates.last().unwrap());
                        schedule_focus(target.clone());
                    }
                    event.prevent_default();
                }
            } else {
                let Some(last) = container_tab_candidates.last() else {
                    event.prevent_default();
                    return;
                };

                if target == last {
                    let position = body_tab_candidates
                        .iter()
                        .position(|v| v == target)
                        .unwrap();

                    if position == body_tab_candidates.len() {
                        let first = container_tab_candidates.first().unwrap();
                        schedule_focus(first.clone());
                    } else {
                        let target = body_tab_candidates
                            .get(position + 1)
                            .unwrap_or_else(|| body_tab_candidates.first().unwrap());
                        schedule_focus(target.clone());
                    }
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

/// An instance of focus trap.
///
/// This struct contains a handle(Rc) to actual state.
///
/// Dropping this struct would also unregister all event listeners the trap has attached
#[derive(Clone)]
pub struct FocusTrap {
    state: Rc<Mutex<State>>,
}

impl FocusTrap {
    /// Returns the handle to options that were used to construct the trap
    ///
    /// This function locks the state
    pub fn options(&self) -> Rc<FocusTrapOptions> {
        self.state.lock().unwrap().options.clone()
    }

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
            let Some(event) = event.dyn_ref() else {
                return;
            };
            state.handle_key_down(event);
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
