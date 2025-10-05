use std::{collections::VecDeque, ops::Deref, rc::Rc};

use gloo::{console::info, utils::document};
use parking_lot::{Mutex, MutexGuard};
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, ResizeObserver};

use crate::{DismissReason, ToastEvent, ToastHandle, Toaster};

/// Instance of rendered toast
#[derive(Clone)]
struct Rendered {
    handle: ToastHandle,
    element: HtmlElement,
}

struct Impl {
    toaster: Toaster,
    container: HtmlElement,
    rendered: Mutex<VecDeque<Rendered>>,
    options: RendererOptions,
}

pub struct RendererOptions {
    /// Gap between rendered toasts
    pub gap: i32,
    /// Max visible toasts at the time
    pub visible: usize,
}

impl Default for RendererOptions {
    fn default() -> Self {
        Self {
            gap: 14,
            visible: 3,
        }
    }
}

#[derive(Clone)]
pub struct Renderer(Rc<Impl>);

impl Renderer {
    fn initialize(&self) {
        self.0
            .container
            .set_attribute("data-seigi-toaster", "")
            .unwrap();

        let callback = Box::new({
            let this = self.clone();

            move |v: &ToastEvent| match v {
                ToastEvent::Create { handle } => {
                    this.on_toast_create(*handle);
                }
                ToastEvent::Update { handle: _ } => todo!(),
                ToastEvent::Dismiss { handle, reason } => {
                    this.on_toast_dismiss(*handle, reason.clone())
                }
            }
        });
        self.0.toaster.subscribe(callback);
    }

    fn on_toast_create(&self, handle: ToastHandle) {
        let toast = self.0.toaster.get(handle).unwrap();

        let element = document().create_element("li").unwrap();
        element.set_attribute("data-seigi-toast", "").unwrap();
        element
            .append_child(
                document()
                    .create_text_node(toast.title.as_str())
                    .unchecked_ref(),
            )
            .unwrap();
        self.0
            .container
            .append_child(element.unchecked_ref())
            .unwrap();

        self.0.rendered.lock().push_front(Rendered {
            handle,
            element: element.unchecked_into(),
        });

        self.update_transforms();
    }

    fn on_toast_dismiss(&self, handle: ToastHandle, _reason: DismissReason) {
        let Some(position) = self
            .0
            .rendered
            .lock()
            .iter()
            .position(|v| v.handle == handle)
        else {
            return;
        };

        let Some(rendered) = self.0.rendered.lock().remove(position) else {
            return;
        };

        let element = rendered.element;
        let _ = element.set_attribute("data-dismissed", "");
        let _ = element.remove_attribute("data-visible");

        self.update_transforms();
    }

    fn update_transforms(&self) {
        // Clone indices to avoid locking
        let indices = {
            let guard = self.0.rendered.lock();
            guard.clone()
        };

        // summed heights until now
        let mut heights_offset = 0;
        for (index, rendered) in indices.iter().enumerate() {
            let element = &rendered.element;
            let _ = element.set_attribute("data-offset", format!("{heights_offset}").as_str());

            if index < self.0.options.visible - 1 {
                heights_offset += element.offset_height() + self.0.options.gap;
            }

            let _ = element.set_attribute("data-visible", "");

            if index >= self.0.options.visible {
                let _ = element.set_attribute(
                    "data-collapsed",
                    format!("{}", index - self.0.options.visible).as_str(),
                );
            } else {
                let _ = element.remove_attribute("data-collapsed");
            }
        }
    }
}

pub fn create_renderer(
    toaster: Toaster,
    container: HtmlElement,
    options: RendererOptions,
) -> Renderer {
    let renderer = Renderer(Rc::new(Impl {
        toaster,
        container,
        rendered: Mutex::new(VecDeque::new()),
        options,
    }));

    renderer.initialize();

    renderer
}
