use gloo::{events::EventListener, utils::document};
use seigi::{
    focus::{FocusTrapHooks, FocusTrapOptions},
    toast::Toast,
};
use wasm_bindgen::JsCast;

use crate::escape_selector;

fn initialize_trap(activate_selector: &str, target_selector: &str) {
    let activate_selector = escape_selector(activate_selector);
    let target_selector = escape_selector(target_selector);

    let activate = document()
        .query_selector(activate_selector.as_str())
        .unwrap()
        .unwrap();
    let target = document()
        .query_selector(target_selector.as_str())
        .unwrap()
        .unwrap();

    let trap = seigi::focus::create(FocusTrapOptions {
        return_focus: true,
        initial_focus: seigi::focus::InitialFocus::Auto,
        deactivate_on_escape: true,
        hooks: FocusTrapHooks {
            activate: Some(Box::new(|| {
                seigi::toast::create_toast(Toast::builder().title("Activated").build());
            })),
            deactivate: Some(Box::new(|| {
                seigi::toast::create_toast(Toast::builder().title("Deactivated").build());
            })),
        },
        target: target.clone().unchecked_into(),
    });

    EventListener::new(activate.clone().unchecked_ref(), "click", move |event| {
        if trap.is_activated() {
            trap.deactivate();
            target.remove_attribute("data-seigi-trap-active");
        } else {
            trap.activate();
            target.set_attribute("data-seigi-trap-active", "");
        }
    })
    .forget();
}

pub fn initialize() {
    initialize_trap("#focus.trap.default.activate", "#focus.trap.default");
    initialize_trap("#focus.trap.esc.activate", "#focus.trap.esc");
}
