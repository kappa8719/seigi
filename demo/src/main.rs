use gloo::console::console_dbg;

mod router;
mod toast;

fn main() {
    console_error_panic_hook::set_once();
    router::initialize();
    seigi::toast::initialize();
    toast::initialize();
}
