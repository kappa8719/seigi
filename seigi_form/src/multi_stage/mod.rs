//! Headless multi staged form with support of user visuals

use std::{
    rc::{Rc, Weak},
    sync::Mutex,
};

use seigi_focus::{FocusTrap, FocusTrapOptions};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{HtmlElement, ResizeObserver};

/// A instance of stage of a form
pub struct Stage {
    container: HtmlElement,
}

impl Stage {
    /// Creates a stage from given container element
    pub fn from_container(container: HtmlElement) -> Self {
        Self { container }
    }
}

/// Actual implementation of [Form]
struct Inner {
    container: HtmlElement,
    stages: Vec<Stage>,
    traps: Vec<FocusTrap>,
    resize_observer: ResizeObserver,
    current: usize,
    is_activated: bool,
    is_locked: bool,
}

impl Inner {
    fn new(
        this: Weak<Mutex<Self>>,
        container: HtmlElement,
        stages: Vec<Stage>,
        traps: Vec<FocusTrap>,
        current: usize,
    ) -> Self {
        Self {
            stages,
            container,
            traps,
            resize_observer: Self::create_resize_observer(this),
            current,
            is_activated: false,
            is_locked: false,
        }
    }

    fn create_resize_observer(this: Weak<Mutex<Self>>) -> ResizeObserver {
        let closure: Closure<dyn Fn()> = Closure::new(move || {
            let this = this.upgrade().unwrap();
            this.lock().unwrap().update_meta();
        });

        let resize_observer = ResizeObserver::new(closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();

        resize_observer
    }

    fn update_relatives(&mut self) {
        for (index, stage) in self.stages.iter().enumerate() {
            let relative = index as isize - self.current as isize;
            let _ = stage
                .container
                .set_attribute("data-seigi-stage-relative", relative.to_string().as_str());
        }
    }

    fn update_meta(&mut self) {
        let stage = &self.stages[self.current].container;

        let _ = self.container.set_attribute(
            "data-seigi-form-width",
            stage.offset_width().to_string().as_str(),
        );
        let _ = self.container.set_attribute(
            "data-seigi-form-height",
            stage.offset_height().to_string().as_str(),
        );
        let _ = self.container.set_attribute(
            "data-seigi-form-offset-x",
            (stage.offset_left()).to_string().as_str(),
        );
        let _ = self.container.set_attribute(
            "data-seigi-form-offset-y",
            (stage.offset_top()).to_string().as_str(),
        );
    }

    fn update_stage(&mut self, target: usize) {
        if self.is_locked || !self.is_activated {
            return;
        }

        self.traps.get(self.current).unwrap().deactivate();
        self.traps.get(target).unwrap().activate();
        self.resize_observer
            .unobserve(self.stages[self.current].container.unchecked_ref());
        self.resize_observer
            .observe(self.stages[target].container.unchecked_ref());

        self.current = target;
        self.update_relatives();
    }

    fn activate(&mut self) {
        if self.is_activated {
            return;
        }
        self.is_activated = true;

        self.traps.get(self.current).unwrap().activate();
        self.resize_observer
            .observe(self.stages[self.current].container.unchecked_ref());

        let _ = self.container.set_attribute("data-seigi-form-active", "");

        self.update_relatives();
    }

    fn deactivate(&mut self) {
        if !self.is_activated {
            return;
        }
        self.is_activated = false;

        self.traps.get(self.current).unwrap().deactivate();
        self.resize_observer
            .unobserve(self.stages[self.current].container.unchecked_ref());

        let _ = self.container.remove_attribute("data-seigi-form-active");
    }
}

/// An instance of multi staged form
///
/// This struct internally contains a handle(Rc) to actual data, so cloning this struct is a
/// lightweight operation
///
/// # Attributes
/// **data-seigi-form-active** is set in the root container if the form is activated and removed if
/// the form is deactivated
///
/// **data-seigi-form-width** is set in the root container to the width of current stage container in
/// px
///
/// **data-seigi-form-height** is set in the root container to the height of current stage container
/// in px
///
/// **data-seigi-form-offset-x** is set in the root container to the sum of widths of previous
/// stages
///
/// **data-seigi-form-offset-y** is set in the root container to the sum of heights of previous
/// stages
///
/// **data-seigi-stage-relative** is set in the each stage containers to the relative index from
/// current stage. For example, a stage currently active has this value of 0, the previous one is
/// -1, and the next one is 1
#[derive(Clone)]
pub struct Form(Rc<Mutex<Inner>>);

impl Form {
    pub fn builder() -> FormBuilder {
        FormBuilder::new()
    }

    /// Updates the current stage to next stage
    pub fn next(&self) {
        let mut inner = self.0.lock().unwrap();
        let current = inner.current;
        inner.update_stage(current + 1);
    }

    /// Updates the current stage to previous stage
    pub fn previous(&self) {
        let mut inner = self.0.lock().unwrap();
        let current = inner.current;
        inner.update_stage(current - 1);
    }

    /// Updates the current stage
    pub fn stage(&self, stage: usize) {
        self.0.lock().unwrap().update_stage(stage);
    }

    /// Returns the current stage
    pub fn current(&self) -> usize {
        self.0.lock().unwrap().current
    }

    /// Activate the form
    pub fn activate(&self) {
        self.0.lock().unwrap().activate();
    }

    /// Deactivate the form
    pub fn deactivate(&self) {
        self.0.lock().unwrap().deactivate();
    }
}

/// A builder struct for [Form]
pub struct FormBuilder {
    initial_stage: usize,
    container: Option<HtmlElement>,
    stages: Vec<Stage>,
}

impl FormBuilder {
    /// Creates a new [FormBuilder]
    pub fn new() -> Self {
        Self {
            initial_stage: 0,
            container: None,
            stages: vec![],
        }
    }

    /// Sets initial stage index for the form
    pub fn initial_stage(mut self, initial_stage: usize) -> Self {
        self.initial_stage = initial_stage;
        self
    }

    /// Sets container element for the form
    pub fn container(mut self, container: HtmlElement) -> Self {
        self.container = Some(container);
        self
    }

    /// Adds a stage to the form
    pub fn add_stage(mut self, stage: Stage) -> Self {
        self.stages.push(stage);
        self
    }

    /// Adds multiple stages to the form
    pub fn add_stages(mut self, stages: impl Iterator<Item = Stage>) -> Self {
        self.stages.extend(stages);
        self
    }

    pub fn build(self) -> Form {
        if self.initial_stage >= self.stages.len() {
            panic!("initial_stage must be less than stage count");
        }

        let container = self.container.expect("container must be set to build Form");

        let traps = self
            .stages
            .iter()
            .map(|v| {
                seigi_focus::create(
                    FocusTrapOptions::builder()
                        .return_focus(false)
                        .deactivate_on_escape(false)
                        .scope(container.clone().unchecked_into())
                        .target(v.container.clone())
                        .build(),
                )
            })
            .collect();

        Form(Rc::new_cyclic(|weak| {
            Mutex::new(Inner::new(
                weak.clone(),
                container,
                self.stages,
                traps,
                self.initial_stage,
            ))
        }))
    }
}

impl Default for FormBuilder {
    fn default() -> Self {
        Self::new()
    }
}
