//! Focus management with accessibility

use std::{rc::Rc, sync::Mutex};

use gloo::{
    console::info,
    events::{EventListener, EventListenerOptions},
    timers::callback::Timeout,
    utils::document,
};
use wasm_bindgen::JsCast;
use web_sys::{Element, Event, FocusEvent, HtmlElement, KeyboardEvent, MouseEvent};

const CANDIDATE_SELECTOR: &str = "input:not([inert]),\
    select:not([inert]),\
    textarea:not([inert]),\
    a[href]:not([inert]),\
    button:not([inert]),\
    [tabindex]:not(slot):not([inert]),\
    audio[controls]:not([inert]),\
    video[controls]:not([inert]),\
    [contenteditable]:not([contenteditable=\"false\"]):not([inert]),\
    details>summary:first-of-type:not([inert]),\
    details:not([inert])";

/// A macro for building an event listener
macro_rules! event_listener {
    ($target: expr, $event_type: literal, $captures: literal, $clone: ident, $callback: expr) => {
        EventListener::new_with_options(
            $target.unchecked_ref(),
            $event_type,
            if $captures {
                EventListenerOptions {
                    phase: gloo::events::EventListenerPhase::Capture,
                    passive: false,
                }
            } else {
                EventListenerOptions::enable_prevent_default()
            },
            {
                let $clone = $clone.clone();
                $callback
            },
        )
    };
}

fn is_disabled(element: &Element) -> bool {
    if let Some(disabled) = element.get_attribute("disabled") {
        return match disabled.as_str() {
            "true" => true,
            &_ => false,
        };
    }

    false
}

fn is_inert(element: &Element) -> bool {
    if let Some(inert) = element.get_attribute("inert") {
        return inert == "true";
    }

    false
}

fn has_inert_ancestor(element: &Element) -> bool {
    let Some(mut current) = element.parent_element() else {
        return false;
    };
    loop {
        if is_inert(element) {
            return true;
        }

        if let Some(parent) = current.parent_element() {
            current = parent;
        } else {
            return false;
        }
    }
}

fn is_hidden_input(element: &Element) -> bool {
    if element.tag_name() == "input"
        && let Some(t) = element.get_attribute("type")
        && t == "hidden"
    {
        return true;
    }

    false
}

pub fn is_focusable(element: &HtmlElement) -> bool {
    if is_disabled(element)
        || is_inert(element)
        || has_inert_ancestor(element)
        || is_hidden_input(element)
    {
        return false;
    }

    true
}

pub fn is_tabbable(element: &HtmlElement) -> bool {
    if element.tab_index() < 0 || !is_focusable(element) {
        return false;
    }

    true
}

fn candidates(container: &Element, filter: impl Fn(&HtmlElement) -> bool) -> Vec<HtmlElement> {
    let Ok(elements) = container.query_selector_all(CANDIDATE_SELECTOR) else {
        return vec![];
    };

    let mut candidates = Vec::with_capacity(elements.length() as usize);
    for element in elements.values() {
        let Ok(element) = element else {
            continue;
        };

        let Ok(element) = element.dyn_into::<HtmlElement>() else {
            continue;
        };

        if !filter(&element) {
            continue;
        }

        candidates.push(element);
    }

    candidates
}

fn first_candidate(
    container: &Element,
    filter: impl Fn(&HtmlElement) -> bool,
) -> Option<HtmlElement> {
    let Ok(elements) = container.query_selector_all(CANDIDATE_SELECTOR) else {
        return None;
    };

    for element in elements.values() {
        let Ok(element) = element else {
            continue;
        };

        let Ok(element) = element.dyn_into::<HtmlElement>() else {
            continue;
        };

        if filter(&element) {
            return Some(element);
        }
    }

    None
}

pub fn tab_candidates(container: &Element) -> Vec<HtmlElement> {
    candidates(container, is_tabbable)
}

pub fn focus_candidates(container: &Element) -> Vec<HtmlElement> {
    candidates(container, is_focusable)
}

#[allow(unused)]
struct FocusTrapListeners {
    focus_in: EventListener,
    mouse_down: EventListener,
    touch_start: EventListener,
    click: EventListener,
    key_down_capture: EventListener,
    key_down: EventListener,
}

pub struct FocusTrapOptions {
    /// Returns focus to last focused element while deactivated on deactivate
    pub return_focus_on_deactivate: bool,
    /// Blocks interactions to outside if true
    pub trap_interactions: bool,
    /// Deactivate on escape key pressed
    pub deactivate_on_escape: bool,
    /// The container to which the trap is attached
    pub target: Element,
}

impl FocusTrapOptions {
    pub fn new(target: Element) -> FocusTrapOptions {
        FocusTrapOptions {
            return_focus_on_deactivate: true,
            trap_interactions: true,
            deactivate_on_escape: true,
            target,
        }
    }
}

struct FocusTrapState {
    options: Rc<FocusTrapOptions>,
    /// Is the focus trap activated
    activated: bool,
    /// Attachted event listeners
    listeners: Option<FocusTrapListeners>,
    /// The last focused element
    last_focus: Option<HtmlElement>,
    /// The element to which trap should focus when deactivated
    return_to: Option<HtmlElement>,
}

#[derive(Clone)]
pub struct FocusTrap {
    options: Rc<FocusTrapOptions>,
    state: Rc<Mutex<FocusTrapState>>,
}

impl FocusTrap {
    /// Return the activation of the trap
    pub fn is_activated(&self) -> bool {
        self.state.lock().unwrap().activated
    }

    /// Activates the trap
    ///
    /// Does nothing if the trap is already activated
    pub fn activate(&self) {
        {
            let mut state = self.state.lock().unwrap();
            if state.activated {
                return;
            }
            state.activated = true;

            let current = document()
                .active_element()
                .map(|v| v.dyn_into::<HtmlElement>().ok())
                .unwrap();
            state.return_to = current;

            if state.should_initial_focus() {
                state.initial_focus();
            }
        }

        self.add_listeners();
    }

    /// Deactivates the trap
    ///
    /// Does nothing if the trap is already deactivated
    pub fn deactivate(&self) {
        {
            let mut state = self.state.lock().unwrap();
            if !state.activated {
                return;
            }
            state.activated = false;
        }

        // The listeners must be remove before returning focus because
        // they would try to put focus back in to container
        self.remove_listeners();

        {
            let state = self.state.lock().unwrap();
            if state.should_return_focus() {
                state.return_focus();
            }
        }
    }

    fn add_listeners(&self) {
        if self.state.lock().unwrap().listeners.is_some() {
            return;
        }

        let document = document();
        let state = self.state.clone();

        let focus_in = event_listener!(document, "focusin", true, state, move |event| {
            let mut state = state.lock().expect("failed to acquire lock");
            state.handle_focus_in(event.unchecked_ref());
        });
        let mouse_down = event_listener!(document, "mousedown", true, state, move |event| {
            let state = state.lock().expect("failed to acquire lock");
            state.handle_pointer_down(event.unchecked_ref());
        });
        let touch_start = event_listener!(document, "touchstart", true, state, move |event| {
            let state = state.lock().expect("failed to acquire lock");
            state.handle_pointer_down(event.unchecked_ref());
        });
        let click = event_listener!(document, "click", true, state, move |event| {
            let state = state.lock().expect("failed to acquire lock");
            state.handle_click(event.unchecked_ref());
        });
        let key_down_capture = event_listener!(document, "keydown", true, state, move |event| {
            let state = state.lock().expect("failed to acquire lock");
            state.handle_key_down(event.unchecked_ref());
        });
        let key_down = event_listener!(document, "keydown", false, state, move |event| {
            let state = state.lock().expect("failed to acquire lock");
            state.handle_key_down(event.unchecked_ref());
        });

        let listeners = FocusTrapListeners {
            focus_in,
            mouse_down,
            touch_start,
            click,
            key_down_capture,
            key_down,
        };

        let mut state = self.state.lock().unwrap();
        state.listeners = Some(listeners);
    }

    fn remove_listeners(&self) {
        let mut state = self.state.lock().unwrap();

        // Just drop the listeners because EventListener is automatically removed when dropped
        state.listeners.take();
    }
}

impl FocusTrapState {
    fn get_tab_candidates(&self) -> Vec<HtmlElement> {
        tab_candidates(&self.options.target)
    }

    fn get_focus_candidates(&self) -> Vec<HtmlElement> {
        focus_candidates(&self.options.target)
    }

    fn get_first_tab_candidate(&self) -> Option<HtmlElement> {
        first_candidate(&self.options.target, is_tabbable)
    }

    fn get_first_focus_candidate(&self) -> Option<HtmlElement> {
        first_candidate(&self.options.target, is_focusable)
    }

    fn should_initial_focus(&self) -> bool {
        let Some(current) = document().active_element() else {
            return false;
        };

        if self.options.target.contains(Some(&current)) {
            return false;
        }

        true
    }

    /// Find element that must be focused when trap is activated and focus it
    fn initial_focus(&self) {
        let Some(candidate) = self.get_first_focus_candidate() else {
            return;
        };

        Self::async_focus(candidate);
    }

    fn should_return_focus(&self) -> bool {
        if !self.options.return_focus_on_deactivate {
            return false;
        }

        true
    }

    /// Focus [Self::return_to] element
    fn return_focus(&self) {
        if let Some(return_to) = self.return_to.clone() {
            Self::async_focus(return_to);
        }
    }

    fn async_focus(element: HtmlElement) {
        Timeout::new(0, move || {
            let _ = element.focus();
        })
        .forget();
    }

    fn handle_focus_in(&mut self, event: &FocusEvent) {
        let Some(target) = event.target() else {
            return;
        };

        let Ok(target) = target.dyn_into::<HtmlElement>() else {
            return;
        };

        if self.options.target.contains(Some(&target)) {
            self.last_focus = Some(target)
        } else {
            // the focus has escaped out of focus trap
            event.stop_immediate_propagation();

            if let Some(last_focus) = &self.last_focus {
                Self::async_focus(last_focus.clone());
            }
        }
    }

    fn handle_pointer_down(&self, _event: &Event) {
        // if !self.options.trap_interactions {
        //     return;
        // }
    }

    fn handle_click(&self, _event: &MouseEvent) {}

    fn handle_key_down(&self, event: &KeyboardEvent) {
        if event.key() == "Tab" {
            let Some(target) = event.target() else {
                return;
            };
            let target = target.unchecked_ref::<HtmlElement>();
            let is_backward = event.shift_key();
            let tab_candidates = self.get_tab_candidates();

            if is_backward {
                let Some(first) = tab_candidates.first() else {
                    event.prevent_default();
                    return;
                };

                if target == first {
                    // If there was a first element in vec, then there must be last one too
                    let last = tab_candidates.last().unwrap();
                    Self::async_focus(last.clone());
                    event.prevent_default();
                }
            } else {
                let Some(last) = tab_candidates.last() else {
                    event.prevent_default();
                    return;
                };

                if target == last {
                    let first = tab_candidates.first().unwrap();
                    Self::async_focus(first.clone());
                    event.prevent_default();
                }
            }
        } else if event.key() == "Escape" {
            if self.options.deactivate_on_escape {
                event.prevent_default();
            }
        }
    }
}

pub fn create_focus_trap(options: FocusTrapOptions) -> FocusTrap {
    let options = Rc::new(options);
    let state = Rc::new(Mutex::new(FocusTrapState {
        options: options.clone(),
        activated: false,
        listeners: None,
        last_focus: None,
        return_to: None,
    }));

    FocusTrap { options, state }
}
