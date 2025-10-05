use std::{
    cell::RefCell,
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use gloo::{console::console_dbg, events::EventListener, utils::document};
use seigi::toast::Toast;
use wasm_bindgen::JsCast;

pub fn initialize() {
    let create_button = document()
        .query_selector("#toast\\.create")
        .unwrap()
        .unwrap();

    let dismiss_button = document()
        .query_selector("#toast\\.dismiss")
        .unwrap()
        .unwrap();

    let sequence = RefCell::new(0usize);
    let toasts = Arc::new(Mutex::new(VecDeque::new()));

    EventListener::new(create_button.unchecked_ref(), "click", {
        let toasts = toasts.clone();
        move |_| {
            let current = sequence.replace_with(|v| *v + 1);
            let mut toasts = toasts.lock().unwrap();
            toasts.push_front(seigi::toast::create_toast(
                Toast::builder()
                    .title(format!("Toast {current}"))
                    .description("Description")
                    .build(),
            ));
        }
    })
    .forget();

    EventListener::new(dismiss_button.unchecked_ref(), "click", move |_| {
        let mut toasts = toasts.lock().unwrap();
        let Some(last) = toasts.pop_back() else {
            toasts.push_front(seigi::toast::create_toast(
                Toast::builder()
                    .title(format!("No toast to dismiss"))
                    .build(),
            ));
            return;
        };

        seigi::toast::dismiss_toast(last);
    })
    .forget();
}
