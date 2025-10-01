use anyhow::anyhow;
use gloo::{
    console::{console, console_dbg, log},
    events::{self, EventListener, EventListenerOptions},
    utils::{document, history, window},
};
use js_sys::Function;
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{HtmlAnchorElement, HtmlElement, Location, PopStateEvent, Url};

pub fn initialize() {
    let options = EventListenerOptions::enable_prevent_default();

    EventListener::new_with_options(window().unchecked_ref(), "popstate", options, |event| {
        let event = event.dyn_ref::<PopStateEvent>().unwrap();
        let location = document().location().unwrap();
        let path = location.pathname().unwrap();
        update_route_transitioning(path.as_str());
    })
    .forget();

    EventListener::new_with_options(window().unchecked_ref(), "click", options, |event| {
        let Some(target) = event.target() else {
            return;
        };

        let Ok(target) = target.dyn_into::<HtmlAnchorElement>() else {
            return;
        };

        event.prevent_default();

        let to = target.href();
        let url = Url::new(to.as_str()).unwrap();
        let pathname = url.pathname();

        history().push_state_with_url(
            js_sys::Object::new().unchecked_ref(),
            "",
            Some(pathname.as_str()),
        );
        update_route_transitioning(pathname.as_str());
    })
    .forget();
}

fn update_route_transitioning(url: &str) {
    fn update_route_logging(url: &str) {
        if let Err(result) = update_route(url) {
            log!(format!("failed to update route: {result:?}"));
        }
    }
    let callback: Closure<dyn Fn()> = Closure::new({
        let url = url.to_string();
        move || {
            update_route_logging(url.as_str());
        }
    });
    if let Err(_) = document()
        .start_view_transition_with_update_callback(Some(callback.as_ref().unchecked_ref()))
    {
        update_route_logging(url);
    };
    callback.forget();
}

fn update_route(url: &str) -> anyhow::Result<()> {
    let Ok(routes) = document().query_selector_all("[data-route]") else {
        return Err(anyhow!("failed to retrieve routes"));
    };

    for element in routes.values() {
        let Ok(element) = element else {
            continue;
        };

        let Ok(element) = element.dyn_into::<HtmlElement>() else {
            continue;
        };

        let Some(route) = element.get_attribute("data-route") else {
            continue;
        };

        let active = route == url;
        if active {
            let _ = element.set_attribute("data-route-active", "");
        } else {
            let _ = element.remove_attribute("data-route-active");
        }
    }

    Ok(())
}
