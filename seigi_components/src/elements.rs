use wasm_bindgen::{JsValue, convert::FromWasmAbi};
use web_sys::*;

pub mod constructors {
    use wasm_bindgen::prelude::*;

    macro_rules! constructor {
        ($t: ident, $binding: literal) => {
            #[wasm_bindgen]
            #[allow(non_upper_case_globals)]
            extern "C" {
                #[wasm_bindgen(thread_local_v2, js_name = $binding, js_namespace = window)]
                pub static $t: js_sys::Function;
            }
        };
    }

    macro_rules! constructors {
        ( $($t: ident => $binding: literal),* $(,)? ) => {
            $(
                constructor!($t, $binding);
            )*
        };
    }

    constructors! {
        HtmlAnchorElement => "HTMLAnchorElement",
        HtmlAreaElement => "HTMLAreaElement",
        HtmlAudioElement => "HTMLAudioElement",
        HtmlBaseElement => "HTMLBaseElement",
        HtmlBodyElement => "HTMLBodyElement",
        HtmlBrElement => "HTMLBRElement",
        HtmlButtonElement => "HTMLButtonElement",
        HtmlCanvasElement => "HTMLCanvasElement",
        HtmlDListElement => "HTMLDListElement",
        HtmlDataElement => "HTMLDataElement",
        HtmlDataListElement => "HTMLDataListElement",
        HtmlDetailsElement => "HTMLDetailsElement",
        HtmlDialogElement => "HTMLDialogElement",
        HtmlDirectoryElement => "HTMLDirectoryElement",
        HtmlDivElement => "HTMLDivElement",
        HtmlElement => "HTMLElement",
        HtmlEmbedElement => "HTMLEmbedElement",
        HtmlFieldSetElement => "HTMLFieldSetElement",
        HtmlFontElement => "HTMLFontElement",
        HtmlFormElement => "HTMLFormElement",
        HtmlFrameElement => "HTMLFrameElement",
        HtmlFrameSetElement => "HTMLFrameSetElement",
        HtmlHeadElement => "HTMLHeadElement",
        HtmlHeadingElement => "HTMLHeadingElement",
        HtmlHrElement => "HTMLHRElement",
        HtmlHtmlElement => "HTMLHtmlElement",
        HtmlIFrameElement => "HTMLIFrameElement",
        HtmlImageElement => "HTMLImageElement",
        HtmlInputElement => "HTMLInputElement",
        HtmlLabelElement => "HTMLLabelElement",
        HtmlLegendElement => "HTMLLegendElement",
        HtmlLiElement => "HTMLLIElement",
        HtmlLinkElement => "HTMLLinkElement",
        HtmlMapElement => "HTMLMapElement",
        HtmlMediaElement => "HTMLMediaElement",
        HtmlMenuElement => "HTMLMenuElement",
        HtmlMetaElement => "HTMLMetaElement",
        HtmlMeterElement => "HTMLMeterElement",
        HtmlModElement => "HTMLModElement",
        HtmlOListElement => "HTMLOListElement",
        HtmlObjectElement => "HTMLObjectElement",
        HtmlOptGroupElement => "HTMLOptGroupElement",
        HtmlOptionElement => "HTMLOptionElement",
        HtmlOutputElement => "HTMLOutputElement",
        HtmlParagraphElement => "HTMLParagraphElement",
        HtmlParamElement => "HTMLParamElement",
        HtmlPictureElement => "HTMLPictureElement",
        HtmlPreElement => "HTMLPreElement",
        HtmlProgressElement => "HTMLProgressElement",
        HtmlQuoteElement => "HTMLQuoteElement",
        HtmlScriptElement => "HTMLScriptElement",
        HtmlSelectElement => "HTMLSelectElement",
        HtmlSlotElement => "HTMLSlotElement",
        HtmlSourceElement => "HTMLSourceElement",
        HtmlSpanElement => "HTMLSpanElement",
        HtmlStyleElement => "HTMLStyleElement",
        HtmlTableCaptionElement => "HTMLTableCaptionElement",
        HtmlTableCellElement => "HTMLTableCellElement",
        HtmlTableColElement => "HTMLTableColElement",
        HtmlTableElement => "HTMLTableElement",
        HtmlTableRowElement => "HTMLTableRowElement",
        HtmlTableSectionElement => "HTMLTableSectionElement",
        HtmlTemplateElement => "HTMLTemplateElement",
        HtmlTextAreaElement => "HTMLTextAreaElement",
        HtmlTimeElement => "HTMLTimeElement",
        HtmlTitleElement => "HTMLTitleElement",
        HtmlTrackElement => "HTMLTrackElement",
        HtmlUListElement => "HTMLUListElement",
        HtmlUnknownElement => "HTMLUnknownElement",
        HtmlVideoElement => "HTMLVideoElement",
    }
}

macro_rules! inheritable {
    ($t: ident, $tag: literal) => {
        impl InheritableElement for $t {
            fn constructor() -> js_sys::Function {
                constructors::$t.with(|v| v.clone())
            }

            fn tag() -> &'static str {
                $tag
            }
        }
    };
}

pub trait InheritableElement: FromWasmAbi + AsRef<JsValue> {
    fn constructor() -> js_sys::Function;
    fn tag() -> &'static str;
}

inheritable!(HtmlElement, "");
inheritable!(HtmlAnchorElement, "a");
inheritable!(HtmlAreaElement, "area");
inheritable!(HtmlAudioElement, "audio");
inheritable!(HtmlBaseElement, "base");
inheritable!(HtmlBodyElement, "body");
inheritable!(HtmlBrElement, "br");
inheritable!(HtmlButtonElement, "button");
inheritable!(HtmlCanvasElement, "canvas");
inheritable!(HtmlDListElement, "dl");
inheritable!(HtmlDataElement, "data");
inheritable!(HtmlDataListElement, "datalist");
inheritable!(HtmlDetailsElement, "details");
inheritable!(HtmlDialogElement, "dialog");
inheritable!(HtmlDirectoryElement, "directory");
inheritable!(HtmlDivElement, "div");
inheritable!(HtmlEmbedElement, "embed");
inheritable!(HtmlFieldSetElement, "fieldset");
inheritable!(HtmlFontElement, "font");
inheritable!(HtmlFormElement, "form");
inheritable!(HtmlFrameElement, "frame");
inheritable!(HtmlFrameSetElement, "frameset");
inheritable!(HtmlHeadElement, "head");
inheritable!(HtmlHeadingElement, "heading");
inheritable!(HtmlHrElement, "hr");
inheritable!(HtmlHtmlElement, "html");
inheritable!(HtmlIFrameElement, "iframe");
inheritable!(HtmlImageElement, "image");
inheritable!(HtmlInputElement, "input");
inheritable!(HtmlLabelElement, "label");
inheritable!(HtmlLegendElement, "legend");
inheritable!(HtmlLiElement, "li");
inheritable!(HtmlLinkElement, "link");
inheritable!(HtmlMapElement, "map");
inheritable!(HtmlMenuElement, "menu");
inheritable!(HtmlMetaElement, "meta");
inheritable!(HtmlMeterElement, "meter");
inheritable!(HtmlOListElement, "ol");
inheritable!(HtmlObjectElement, "object");
inheritable!(HtmlOptGroupElement, "optgroup");
inheritable!(HtmlOptionElement, "option");
inheritable!(HtmlOutputElement, "output");
inheritable!(HtmlParagraphElement, "p");
inheritable!(HtmlParamElement, "param");
inheritable!(HtmlPictureElement, "picture");
inheritable!(HtmlPreElement, "pre");
inheritable!(HtmlProgressElement, "progress");
inheritable!(HtmlQuoteElement, "blackquote");
inheritable!(HtmlScriptElement, "script");
inheritable!(HtmlSelectElement, "select");
inheritable!(HtmlSlotElement, "slot");
inheritable!(HtmlSourceElement, "source");
inheritable!(HtmlSpanElement, "span");
inheritable!(HtmlStyleElement, "style");
inheritable!(HtmlTableCaptionElement, "caption");
inheritable!(HtmlTableElement, "table");
inheritable!(HtmlTemplateElement, "template");
inheritable!(HtmlTextAreaElement, "textarea");
inheritable!(HtmlTimeElement, "time");
inheritable!(HtmlTitleElement, "title");
inheritable!(HtmlTrackElement, "track");
inheritable!(HtmlUListElement, "ul");
inheritable!(HtmlVideoElement, "video");
