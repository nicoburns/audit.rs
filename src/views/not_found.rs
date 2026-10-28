use crate::components::{Center, Content, Error, Footer, SimpleNavbar};
use dioxus::prelude::*;

/// Not found view, shows generic error.
#[component]
pub fn NotFound() -> Element {
    rsx! {
        div { class: "flex flex-col min-h-screen",
            div { class: "flex-1",
                SimpleNavbar {}
                Content {
                    Center {
                        Error { title: "Not found", status: "The URL was not found" }
                    }
                }
            }
            Footer {}
        }
    }
}
