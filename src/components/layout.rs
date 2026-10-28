use dioxus::prelude::*;

#[component]
pub fn Center(children: Element) -> Element {
    rsx! {
        div { style: "position: absolute; top: 50%; width: 100%; text-align: center;", {children} }
    }
}

#[component]
pub fn Content(children: Element) -> Element {
    rsx! {
        div { class: "grow", {children} }
    }
}
