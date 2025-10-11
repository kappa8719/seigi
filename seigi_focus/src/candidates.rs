use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

const CANDIDATE_SELECTOR: &str = "input:not([inert]),\
    select:not([inert]),\
    textarea:not([inert]),\
    a[href]:not([inert]),\
    button:not([inert]),\
    [tabindex]:not(slot):not([inert]),\
    audio[controls]:not([inert]),\
    video[controls]:not([inert]),\
    [contenteditable]:not([contenteditable=\"false\"]):not([inert]),\
    details>summary:first-of-type:not([inert]),\
    details:not([inert])";

fn is_disabled(element: &Element) -> bool {
    if let Some(disabled) = element.get_attribute("disabled") {
        return match disabled.as_str() {
            "true" => true,
            &_ => false,
        };
    }

    false
}

fn is_inert(element: &Element) -> bool {
    if let Some(inert) = element.get_attribute("inert") {
        return inert == "true";
    }

    false
}

fn has_inert_ancestor(element: &Element) -> bool {
    let Some(mut current) = element.parent_element() else {
        return false;
    };
    loop {
        if is_inert(element) {
            return true;
        }

        if let Some(parent) = current.parent_element() {
            current = parent;
        } else {
            return false;
        }
    }
}

fn is_hidden_input(element: &Element) -> bool {
    if element.tag_name() == "input"
        && let Some(t) = element.get_attribute("type")
        && t == "hidden"
    {
        return true;
    }

    false
}

pub fn is_focusable(element: &HtmlElement) -> bool {
    if is_disabled(element)
        || is_inert(element)
        || has_inert_ancestor(element)
        || is_hidden_input(element)
    {
        return false;
    }

    true
}

pub fn is_tabbable(element: &HtmlElement) -> bool {
    if element.tab_index() < 0 || !is_focusable(element) {
        return false;
    }

    true
}
pub fn candidates(container: &Element, filter: impl Fn(&HtmlElement) -> bool) -> Vec<HtmlElement> {
    let Ok(elements) = container.query_selector_all(CANDIDATE_SELECTOR) else {
        return vec![];
    };

    let mut candidates = Vec::with_capacity(elements.length() as usize);
    for element in elements.values() {
        let Ok(element) = element else {
            continue;
        };

        let Ok(element) = element.dyn_into::<HtmlElement>() else {
            continue;
        };

        if !filter(&element) {
            continue;
        }

        candidates.push(element);
    }

    candidates
}

fn first_candidate(
    container: &Element,
    filter: impl Fn(&HtmlElement) -> bool,
) -> Option<HtmlElement> {
    let Ok(elements) = container.query_selector_all(CANDIDATE_SELECTOR) else {
        return None;
    };

    for element in elements.values() {
        let Ok(element) = element else {
            continue;
        };

        let Ok(element) = element.dyn_into::<HtmlElement>() else {
            continue;
        };

        if filter(&element) {
            return Some(element);
        }
    }

    None
}

pub fn tab_candidates(container: &Element) -> Vec<HtmlElement> {
    candidates(container, is_tabbable)
}

pub fn focus_candidates(container: &Element) -> Vec<HtmlElement> {
    candidates(container, is_focusable)
}

pub fn first_tab_candidate(container: &Element) -> Option<HtmlElement> {
    first_candidate(container, is_tabbable)
}

pub fn first_focus_candidate(container: &Element) -> Option<HtmlElement> {
    first_candidate(container, is_focusable)
}
