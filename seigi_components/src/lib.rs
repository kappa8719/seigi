#![feature(associated_type_defaults)]

mod elements;

use std::sync::Arc;

use wasm_bindgen::{
    JsCast, JsValue, UnwrapThrowExt,
    prelude::{Closure, wasm_bindgen},
};
use web_sys::{HtmlElement, ShadowRootMode};

use crate::elements::InheritableElement;

pub trait Component: 'static {
    ///
    type Super: InheritableElement = HtmlElement;

    /// Constructs a new instance of component
    fn construct() -> Self;

    /// An html template that is inserted under the element when connected
    ///
    fn template() -> &'static str {
        ""
    }

    /// Attributes that are observed
    fn observed_attributes() -> Vec<&'static str> {
        vec![]
    }

    /// Whether the element should configure shadow DOM.
    ///
    /// # Returns
    /// None if it does not configure, Some(ShadowRootMode) if it configures
    #[allow(unused_variables)]
    fn attach_shadow(self: &Arc<Self>, element: &Self::Super) -> Option<ShadowRootMode> {
        None
    }

    #[allow(unused_variables)]
    fn connected(self: &Arc<Self>, element: &Self::Super) {}

    #[allow(unused_variables)]
    fn disconnected(self: &Arc<Self>, element: &Self::Super) {}

    #[allow(unused_variables)]
    fn adopted(self: &Arc<Self>, element: &Self::Super) {}

    #[allow(unused_variables)]
    fn attribute_changed(
        self: &Arc<Self>,
        element: &Self::Super,
        name: String,
        old: Option<String>,
        new: Option<String>,
    ) {
    }
}

fn reflect_set<T: AsRef<JsValue>, V: AsRef<JsValue>>(
    target: &T,
    field: &str,
    value: &V,
) -> Result<bool, JsValue> {
    js_sys::Reflect::set(target.as_ref(), &JsValue::from_str(field), value.as_ref())
}

pub fn define<T>(tag: &str)
where
    T: Component,
{
    let template = T::template().to_string();

    let constructor: Closure<dyn Fn(T::Super)> = Closure::new(move |this: T::Super| {
        let instance = Arc::new(T::construct());

        let attach_shadow: Closure<dyn FnMut(T::Super) -> Option<ShadowRootMode>> = Closure::new({
            let instance = instance.clone();
            move |element| instance.attach_shadow(&element)
        });
        let connected_callback: Closure<dyn FnMut(T::Super)> = Closure::new({
            let instance = instance.clone();
            move |element| {
                instance.connected(&element);
            }
        });
        let disconnected_callback: Closure<dyn FnMut(T::Super)> = Closure::new({
            let instance = instance.clone();
            move |element| {
                instance.disconnected(&element);
            }
        });
        let adopted_callback: Closure<dyn FnMut(T::Super)> = Closure::new({
            let instance = instance.clone();
            move |element| {
                instance.adopted(&element);
            }
        });
        let attribute_changed_callback: Closure<
            dyn FnMut(T::Super, String, Option<String>, Option<String>),
        > = Closure::new({
            let instance = instance.clone();
            move |element, name, old, new| {
                instance.attribute_changed(&element, name, old, new);
            }
        });

        reflect_set(&this, "_attachShadow", &attach_shadow).unwrap_throw();
        reflect_set(&this, "_connectedCallback", &connected_callback).unwrap_throw();
        reflect_set(&this, "_disconnectedCallback", &disconnected_callback).unwrap_throw();
        reflect_set(&this, "_adoptedCallback", &adopted_callback).unwrap_throw();
        reflect_set(
            &this,
            "_attributeChangedCallback",
            &attribute_changed_callback,
        )
        .unwrap_throw();

        attach_shadow.forget();
        connected_callback.forget();
        disconnected_callback.forget();
        adopted_callback.forget();
        attribute_changed_callback.forget();
    });

    let observed_attributes = T::observed_attributes()
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>();

    let superclass_tag = T::Super::tag();
    let superclass_tag = if superclass_tag.is_empty() {
        Some(superclass_tag.to_string())
    } else {
        None
    };

    construct(
        tag,
        &T::Super::constructor(),
        superclass_tag,
        constructor.as_ref().unchecked_ref(),
        template,
        observed_attributes,
    );

    constructor.forget();
}

#[wasm_bindgen(module = "/src/construct.js")]
extern "C" {
    fn construct(
        tag: &str,
        superclass: &js_sys::Function,
        superclass_tag: Option<String>,
        constructor: &js_sys::Function,
        template: String,
        observed_attributes: Vec<String>,
    );
}
