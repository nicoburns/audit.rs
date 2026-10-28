use crate::components::{Content, Footer, SimpleNavbar};
use dioxus::prelude::*;

const TEXT: &str = include_str!("about.md");

#[component]
fn AboutText() -> Element {
    let html = comrak::markdown_to_html(TEXT, &Default::default());
    rsx! {
        div {
            class: "prose prose-slate dark:prose-invert max-w-2xl m-auto p-4 pt-12",
            dangerous_inner_html: html,
        }
    }
}

/// About page, showing background information on this project.
#[component]
pub fn About() -> Element {
    rsx! {
        div { class: "flex flex-col min-h-screen",
            div { class: "flex-1",
                SimpleNavbar {}
                Content { AboutText {} }
            }

            Footer {}
        }
    }
}
