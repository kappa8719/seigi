use std::{mem::forget, rc::Rc};

use gloo::{console::info, events::EventListener, utils::document};
use seigi::form::multi_stage::Stage;
use wasm_bindgen::JsCast;
use web_sys::{Element, NodeList};

use crate::query_selector;

pub fn initialize() {
    let form = seigi::form::multi_stage::Form::builder()
        .stage(Stage::from_container(
            query_selector("#forms.multi_stage.animated.1").unwrap(),
        ))
        .stage(Stage::from_container(
            query_selector("#forms.multi_stage.animated.2").unwrap(),
        ))
        .stage(Stage::from_container(
            query_selector("#forms.multi_stage.animated.3").unwrap(),
        ))
        .build();

    for node in document()
        .query_selector_all("[data-seigi-form-next]")
        .unwrap()
        .values()
    {
        let node = node.unwrap().unchecked_into::<Element>();
        EventListener::new(node.unchecked_ref(), "click", {
            let form = form.clone();
            move |_| {
                form.next();
            }
        })
        .forget();
    }

    for node in document()
        .query_selector_all("[data-seigi-form-previous]")
        .unwrap()
        .values()
    {
        let node = node.unwrap().unchecked_into::<Element>();
        EventListener::new(node.unchecked_ref(), "click", {
            let form = form.clone();
            move |_| {
                form.previous();
            }
        })
        .forget();
    }

    form.activate();
    forget(form);
}
