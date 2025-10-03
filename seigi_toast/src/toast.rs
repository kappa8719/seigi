use crate::DismissReason;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ToastHandle(pub u32);

pub struct Toast {
    pub title: String,
    pub description: Option<String>,
    /// The reason of this toast being dismissed
    ///
    /// None if this toast is not dismissed
    pub dismiss: Option<DismissReason>,
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
}

impl ToastBuilder {
    pub fn new() -> ToastBuilder {
        Self {
            title: String::new(),
            description: None,
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

    pub fn build(self) -> Toast {
        Toast {
            title: self.title,
            description: self.description,
            dismiss: None,
        }
    }
}
