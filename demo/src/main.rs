use gloo::{console::console_dbg, utils::document};
use seigi::toast::ToasterOptions;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlElement, NodeList};

mod focus;
mod form;
mod router;
mod toast;

fn main() {
    console_error_panic_hook::set_once();
    router::initialize();
    seigi::toast::initialize(ToasterOptions::default());

    toast::initialize();
    focus::initialize();
    form::initialize();
}

fn escape_selector(selector: &str) -> String {
    selector.replace(".", "\\.")
}

fn query_selector(selector: &str) -> Option<HtmlElement> {
    document()
        .query_selector(escape_selector(selector).as_str())
        .ok()
        .flatten()
        .map(|v| v.unchecked_into())
}

fn query_selector_all(selector: &str) -> Result<NodeList, JsValue> {
    document().query_selector_all(escape_selector(selector).as_str())
}
