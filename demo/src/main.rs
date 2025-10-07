use gloo::console::console_dbg;
use seigi::toast::ToasterOptions;

mod router;
mod toast;

fn main() {
    console_error_panic_hook::set_once();
    router::initialize();
    seigi::toast::initialize(ToasterOptions::default());
    toast::initialize();
}
