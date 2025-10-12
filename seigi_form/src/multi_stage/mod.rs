use std::{rc::Rc, sync::Mutex};

use gloo::console::info;
use seigi_focus::{FocusTrap, FocusTrapOptions};
use web_sys::HtmlElement;

pub struct Stage {
    container: HtmlElement,
}

impl Stage {
    pub fn from_container(container: HtmlElement) -> Self {
        Self { container }
    }
}

/// Actual implementation of [Form]
struct Inner {
    stages: Vec<Stage>,
    traps: Vec<FocusTrap>,
    is_activated: bool,
    is_locked: bool,
    current: usize,
}

impl Inner {
    fn update_relatives(&mut self) {
        for (index, stage) in self.stages.iter().enumerate() {
            let relative = index as isize - self.current as isize;
            let _ = stage
                .container
                .set_attribute("data-seigi-stage-relative", relative.to_string().as_str());
        }
    }

    fn update_stage(&mut self, target: usize) {
        if self.is_locked || !self.is_activated {
            return;
        }

        self.traps.get(self.current).unwrap().deactivate();
        self.traps.get(target).unwrap().activate();

        self.current = target;
        self.update_relatives();
    }

    fn activate(&mut self) {
        if self.is_activated {
            return;
        }
        self.is_activated = true;

        self.traps.get(self.current).unwrap().activate();
        self.update_relatives();
    }

    fn deactivate(&mut self) {
        if !self.is_activated {
            return;
        }
        self.is_activated = false;

        self.traps.get(self.current).unwrap().deactivate();
    }
}

#[derive(Clone)]
pub struct Form(Rc<Mutex<Inner>>);

impl Form {
    /// Creates a new [FormBuilder]
    pub fn builder() -> FormBuilder {
        FormBuilder::new()
    }

    pub fn next(&self) {
        let mut inner = self.0.lock().unwrap();
        let current = inner.current;
        inner.update_stage(current + 1);
    }

    pub fn previous(&self) {
        let mut inner = self.0.lock().unwrap();
        let current = inner.current;
        inner.update_stage(current - 1);
    }

    pub fn stage(&self, stage: usize) {
        self.0.lock().unwrap().update_stage(stage);
    }

    pub fn current(&self) -> usize {
        self.0.lock().unwrap().current
    }

    pub fn activate(&self) {
        self.0.lock().unwrap().activate();
    }

    pub fn deactivate(&self) {
        self.0.lock().unwrap().deactivate();
    }
}

pub struct FormBuilder {
    initial_stage: usize,
    stages: Vec<Stage>,
}

impl FormBuilder {
    /// Creates a new [FormBuilder]
    pub fn new() -> Self {
        Self {
            initial_stage: 0,
            stages: vec![],
        }
    }

    /// Set initial stage index for the form
    pub fn initial_stage(mut self, initial_stage: usize) -> Self {
        self.initial_stage = initial_stage;
        self
    }

    /// Add the stage to the form
    pub fn stage(mut self, stage: Stage) -> Self {
        self.stages.push(stage);
        self
    }

    pub fn build(self) -> Form {
        if self.initial_stage > self.stages.len() {
            panic!("initial_stage must be less than or equal to stage count");
        }

        let traps = self
            .stages
            .iter()
            .map(|v| {
                seigi_focus::create(FocusTrapOptions {
                    return_focus: false,
                    initial_focus: seigi_focus::InitialFocus::Auto,
                    deactivate_on_escape: false,
                    hooks: seigi_focus::FocusTrapHooks::default(),
                    target: v.container.clone(),
                })
            })
            .collect();

        Form(Rc::new(Mutex::new(Inner {
            stages: self.stages,
            traps,
            is_activated: false,
            is_locked: false,
            current: self.initial_stage,
        })))
    }
}

impl Default for FormBuilder {
    fn default() -> Self {
        Self::new()
    }
}
