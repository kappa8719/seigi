use gloo::{events::EventListener, utils::document};
use seigi::{
    focus::{FocusTrapHooks, FocusTrapOptions, InitialFocus},
    toast::Toast,
};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::escape_selector;

struct Options {
    pub return_focus: bool,
    pub initial_focus: InitialFocus,
    /// Whether trap should deactivate when user press esc
    pub deactivate_on_escape: bool,
}

fn initialize_trap(
    activate_selector: &str,
    deactivate_selector: &str,
    target_selector: &str,
    options: Options,
) {
    let activate_selector = escape_selector(activate_selector);
    let deactivate_selector = escape_selector(deactivate_selector);
    let target_selector = escape_selector(target_selector);

    let activate = document()
        .query_selector(activate_selector.as_str())
        .unwrap()
        .unwrap();
    let deactivate = document()
        .query_selector(deactivate_selector.as_str())
        .unwrap()
        .unwrap();
    let target = document()
        .query_selector(target_selector.as_str())
        .unwrap()
        .unwrap();

    let trap = seigi::focus::create(FocusTrapOptions {
        return_focus: options.return_focus,
        initial_focus: options.initial_focus,
        deactivate_on_escape: options.deactivate_on_escape,
        hooks: FocusTrapHooks {
            activate: Some(Box::new({
                let target = target.clone();
                move || {
                    seigi::toast::create_toast(Toast::builder().title("Activated").build());
                    target.set_attribute("data-seigi-trap-active", "");
                }
            })),
            deactivate: Some(Box::new({
                let target = target.clone();
                move || {
                    seigi::toast::create_toast(Toast::builder().title("Deactivated").build());
                    target.remove_attribute("data-seigi-trap-active");
                }
            })),
        },
        target: target.clone().unchecked_into(),
    });

    EventListener::new(activate.clone().unchecked_ref(), "click", {
        let trap = trap.clone();
        move |event| {
            if trap.is_activated() {
                trap.deactivate();
            } else {
                trap.activate();
            }
        }
    })
    .forget();

    EventListener::new(deactivate.clone().unchecked_ref(), "click", {
        let trap = trap.clone();
        move |event| {
            trap.deactivate();
        }
    })
    .forget();
}

pub fn initialize() {
    initialize_trap(
        "#focus.trap.default.activate",
        "#focus.trap.default.deactivate",
        "#focus.trap.default",
        Options {
            return_focus: true,
            initial_focus: InitialFocus::Auto,
            deactivate_on_escape: false,
        },
    );
    initialize_trap(
        "#focus.trap.esc.activate",
        "#focus.trap.esc.deactivate",
        "#focus.trap.esc",
        Options {
            return_focus: true,
            initial_focus: InitialFocus::Auto,
            deactivate_on_escape: true,
        },
    );
}
