use crate::{components::*, Route};
use dioxus::prelude::*;

#[component]
fn Logo() -> Element {
    rsx! {
        h1 { class: "text-center text-3xl font-bold my-12 dark:text-white", "diff.rs" }
    }
}

/// Home page, shows search bar.
#[component]
pub fn Home() -> Element {
    let navigator = use_navigator();
    let onchange = move |input: String| {
        if !input.is_empty() {
            navigator.push(Route::Search { query: input });
        }
    };
    rsx! {
        div { class: "flex flex-col min-h-screen",
            div { class: "flex-1",
                SimpleNavbar {}
                Content {
                    Logo {}
                    div { class: "max-w-3xl m-auto",
                        SearchBar { value: "", onchange }
                    }
                    DefaultSummarySection {}
                }
            }
            Footer {}
        }
    }
}
