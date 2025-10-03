mod state;
mod toast;
use std::sync::{Mutex, OnceLock};

pub use state::*;
pub use toast::*;

static GLOBAL_TOASTS: OnceLock<Mutex<ToastState>> = OnceLock::new();

/// Initialize global state and renderer
pub fn initialize() {
    // Initialize global state
    GLOBAL_TOASTS.get_or_init(|| Mutex::new(ToastState::new()));
}

/// Add toast to global state
///
/// # Returns
/// Handle to the toast
pub fn create_toast(toast: impl Into<Toast>) -> ToastHandle {
    let toast = toast.into();
    let mut global = GLOBAL_TOASTS.get().unwrap().lock().unwrap();
    global.add_toast(toast)
}

/// Dismiss a toast of handle with given reason from global toast state
///
/// # Returns
/// True if toast has been set to be dismissed, false if no toast of handle was found
pub fn dismiss_toast(handle: ToastHandle, reason: DismissReason) -> bool {
    let mut global = GLOBAL_TOASTS.get().unwrap().lock().unwrap();
    global.dismiss_toast(handle, reason)
}
