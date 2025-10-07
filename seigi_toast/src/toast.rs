use std::time::Duration;

use crate::DismissReason;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ToastHandle(pub u32);

#[derive(Default)]
pub enum ToastTimeout {
    /// No timeout
    None,
    /// Use the default timeout provided by [Toaster]
    #[default]
    Default,
    /// Specified duration
    Duration(Duration),
}

pub struct Toast {
    pub title: String,
    pub description: Option<String>,
    /// The reason of this toast being dismissed
    ///
    /// None if this toast is not dismissed
    pub dismiss: Option<DismissReason>,
    /// The timeout where toast should automatically be dismissed after
    pub timeout: ToastTimeout,
}

impl Toast {
    pub fn builder() -> ToastBuilder {
        ToastBuilder::new()
    }
}

impl From<ToastBuilder> for Toast {
    fn from(value: ToastBuilder) -> Self {
        value.build()
    }
}

#[derive(Default)]
pub struct ToastBuilder {
    title: String,
    description: Option<String>,
    timeout: ToastTimeout,
}

impl ToastBuilder {
    pub fn new() -> ToastBuilder {
        Self {
            title: String::new(),
            description: None,
            timeout: ToastTimeout::default(),
        }
    }

    pub fn title(mut self, title: impl ToString) -> ToastBuilder {
        self.title = title.to_string();
        self
    }

    pub fn description(mut self, description: impl ToString) -> ToastBuilder {
        self.description = Some(description.to_string());
        self
    }

    pub fn timeout(mut self, duration: impl Into<Duration>) -> ToastBuilder {
        self.timeout = ToastTimeout::Duration(duration.into());
        self
    }

    pub fn timeout_secs(self, secs: f64) -> ToastBuilder {
        self.timeout(Duration::from_secs_f64(secs))
    }

    pub fn timeout_default(mut self) -> ToastBuilder {
        self.timeout = ToastTimeout::Default;
        self
    }

    pub fn timeout_none(mut self) -> ToastBuilder {
        self.timeout = ToastTimeout::None;
        self
    }

    pub fn build(self) -> Toast {
        Toast {
            title: self.title,
            description: self.description,
            dismiss: None,
            timeout: self.timeout,
        }
    }
}
