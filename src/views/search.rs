use crate::{components::*, Route};
use dioxus::{core::Task, prelude::*};
use gloo_timers::future::TimeoutFuture;

/// How long the search query has to be stable before results are fetched.
const DEBOUNCE_MILLIS: u32 = 500;

#[component]
fn Logo() -> Element {
    rsx! {
        h1 { class: "text-center text-3xl font-bold my-12 dark:text-white", "diff.rs" }
    }
}

/// Search view, shows search results.
#[component]
pub fn Search(query: String) -> Element {
    let mut state = use_signal(|| query.clone());
    let mut pending = use_signal(|| None::<Task>);
    use_effect(use_reactive!(|query| {
        if let Some(task) = pending.take() {
            task.cancel();
        }
        pending.set(Some(spawn(async move {
            TimeoutFuture::new(DEBOUNCE_MILLIS).await;
            state.set(query);
        })));
    }));

    let navigator = use_navigator();
    let onchange = move |input: String| {
        if input.is_empty() {
            navigator.push(Route::Home {});
        } else {
            navigator.push(Route::Search { query: input });
        }
    };

    rsx! {
        div { class: "flex flex-col min-h-screen",
            div { class: "flex-1",
                SimpleNavbar {}
                Content {
                    div { class: "max-w-3xl m-auto",
                        Logo {}
                        SearchBar { value: query, onchange }
                        div { class: "my-6",
                            SearchResults { query: state() }
                        }
                    }
                }
            }
            Footer {}
        }
    }
}
