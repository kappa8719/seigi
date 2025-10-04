mod renderer;
mod toast;
mod toaster;
use std::cell::OnceCell;

use gloo::utils::{body, document, head};
pub use toast::*;
pub use toaster::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlStyleElement};

use crate::renderer::create_renderer;

thread_local! {
    static GLOBAL_TOASTS: OnceCell<Toaster> = OnceCell::new();
}

fn global() -> Toaster {
    GLOBAL_TOASTS.with(|toaster| toaster.get().unwrap().clone())
}

/// Initialize styles and global
pub fn initialize() {
    initialize_styles();
    initialize_global();
}

/// Add default stylesheet to document head
pub fn initialize_styles() {
    let styles = include_str!("styles.css");
    let element = document()
        .create_element("style")
        .unwrap()
        .unchecked_into::<HtmlStyleElement>();
    head().append_child(element.unchecked_ref()).unwrap();

    element.set_type("text/css");
    element
        .append_child(document().create_text_node(styles).unchecked_ref())
        .unwrap();
}

/// Initialize global state and renderer
pub fn initialize_global() {
    // Initialize global state
    GLOBAL_TOASTS.with(|cell| {
        let toaster = Toaster::new();
        cell.get_or_init(|| toaster.clone());

        let container = document()
            .create_element("ol")
            .unwrap()
            .unchecked_into::<HtmlElement>();
        body().append_child(container.unchecked_ref()).unwrap();
        create_renderer(toaster, container);
    });
}

/// Add toast to global state
///
/// # Returns
/// Handle to the toast
pub fn create_toast(toast: impl Into<Toast>) -> ToastHandle {
    let toast = toast.into();
    global().add_toast(toast)
}

/// Dismiss a toast of handle with given reason from global toast state
///
/// # Returns
/// True if toast has been set to be dismissed, false if no toast of handle was found
pub fn dismiss_toast(handle: ToastHandle, reason: DismissReason) -> bool {
    global().dismiss_toast(handle, reason)
}
