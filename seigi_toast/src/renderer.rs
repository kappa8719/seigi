use gloo::utils::document;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::{Toast, ToastEvent, Toaster};

pub fn create_renderer(state: Toaster, root: HtmlElement) {
    root.set_attribute("data-seigi-toaster", "").unwrap();

    let callback = Box::new({
        let state = state.clone();
        let root = root.clone();

        move |v: &ToastEvent| {
            let root = root.clone();
            match v {
                ToastEvent::Create { handle } => {
                    let toast = state.get(*handle).unwrap();
                    create_toast(root, &toast);
                }
                ToastEvent::Update { handle } => todo!(),
                ToastEvent::Dismiss { handle, reason } => todo!(),
            }
        }
    });
    state.subscribe(callback);
}

fn create_toast(container: HtmlElement, toast: &Toast) {
    let element = document().create_element("li").unwrap();
    element.set_attribute("data-seigi-toast", "").unwrap();
    element
        .append_child(
            document()
                .create_text_node(toast.title.as_str())
                .unchecked_ref(),
        )
        .unwrap();
    container.append_child(element.unchecked_ref()).unwrap();
}
