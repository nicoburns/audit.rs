use dioxus::prelude::*;

#[component]
fn ErrorIcon() -> Element {
    // from https://lucide.dev/icons/circle-x
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            class: "w-12 h-12 text-red-500 dark:text-red-400",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            path { d: "m15 9-6 6" }
            path { d: "m9 9 6 6" }
        }
    }
}

#[component]
fn Spinner() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            class: "animate-spin w-12 h-12 text-blue-500 dark:text-blue-400",
            view_box: "0 0 24 24",
            fill: "none",
            circle {
                class: "opacity-25",
                cx: "12",
                cy: "12",
                r: "10",
                stroke: "currentColor",
                stroke_width: "4",
            }
            path {
                class: "opacity-75",
                fill: "currentColor",
                d: "M4 12a8 8 0 0 1 8-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 0 1 4 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
            }
        }
    }
}

#[component]
pub fn Error(title: String, status: String) -> Element {
    rsx! {
        div { class: "m-auto text-center dark:text-gray-100",
            div { class: "p-2 flex justify-center", ErrorIcon {} }
            div {
                h4 { class: "font-bold p-2", "{title}" }
                div { "{status}" }
            }
        }
    }
}

#[component]
pub fn Loading(title: String, status: String) -> Element {
    rsx! {
        div { class: "m-auto text-center dark:text-gray-100",
            div { class: "p-2 flex justify-center", Spinner {} }
            div {
                h4 { class: "font-bold p-2", "{title}" }
                div { "{status}" }
            }
        }
    }
}
