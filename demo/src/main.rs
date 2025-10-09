use gloo::console::console_dbg;
use seigi::toast::ToasterOptions;

mod focus;
mod router;
mod toast;

fn main() {
    console_error_panic_hook::set_once();
    router::initialize();
    seigi::toast::initialize(ToasterOptions::default());

    toast::initialize();
    focus::initialize();
}

fn escape_selector(selector: &str) -> String {
    selector.replace(".", "\\.")
}
