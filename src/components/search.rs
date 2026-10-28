use crate::{
    cache::SUMMARY_CACHE,
    components::Loading,
    data::{CrateDetail, SearchResponse, SummaryCategory},
    Route,
};
use dioxus::{core::Task, prelude::*};
use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;

/// How long the search input has to be stable before it is reported as changed.
const DEBOUNCE_MILLIS: u32 = 250;

#[component]
pub(super) fn SearchGlass() -> Element {
    rsx! {
        svg {
            class: "w-4 h-4 text-gray-500 dark:text-gray-400",
            "aria-hidden": "true",
            xmlns: "http://www.w3.org/2000/svg",
            fill: "none",
            view_box: "0 0 20 20",
            path {
                stroke: "currentColor",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                stroke_width: "2",
                d: "m19 19-4-4m0-7A7 7 0 1 1 1 8a7 7 0 0 1 14 0Z",
            }
        }
    }
}

#[component]
pub fn SearchBar(value: String, onchange: Option<EventHandler<String>>) -> Element {
    let mut pending = use_signal(|| None::<Task>);
    rsx! {
        form {
            class: "max-w-xl mx-auto p-4",
            // prevent default action on form submission (page reload)
            onsubmit: |event: FormEvent| event.prevent_default(),
            label {
                r#for: "default-search",
                class: "mb-2 text-sm font-medium text-gray-900 sr-only dark:text-white",
                "Search"
            }
            div { class: "relative",
                div { class: "absolute inset-y-0 start-0 flex items-center ps-3 pointer-events-none",
                    SearchGlass {}
                }
                input {
                    r#type: "search",
                    id: "default-search",
                    class: "block w-full p-4 ps-10 text-sm text-gray-900 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
                    placeholder: "Search for crates",
                    required: true,
                    value: "{value}",
                    // set focus
                    onmounted: |event: MountedEvent| async move {
                        let _ = event.set_focus(true).await;
                    },
                    oninput: move |event: FormEvent| {
                        let value = event.value();
                        if let Some(task) = pending.take() {
                            task.cancel();
                        }
                        let task = spawn(async move {
                            TimeoutFuture::new(DEBOUNCE_MILLIS).await;
                            if let Some(onchange) = onchange {
                                onchange.call(value);
                            }
                        });
                        pending.set(Some(task));
                    },
                }
                button {
                    r#type: "submit",
                    class: "text-white absolute end-2.5 bottom-2.5 bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-4 py-2 dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800",
                    "Search"
                }
            }
        }
    }
}

#[component]
pub fn Search() -> Element {
    let mut state = use_signal(String::new);
    let navigator = use_navigator();

    rsx! {
        div { class: "relative w-full",
            div { class: "absolute inset-y-0 start-0 flex items-center ps-3 pointer-events-none",
                SearchGlass {}
            }
            input {
                r#type: "search",
                id: "default-search",
                class: "block w-full p-2 ps-10 text-sm text-gray-900 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-800 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
                placeholder: "Jump to crate...",
                value: "{state}",
                oninput: move |event: FormEvent| state.set(event.value()),
                onkeydown: move |event: KeyboardEvent| {
                    if event.key() == Key::Named(NamedKey::Enter) {
                        navigator.push(Route::Crate { krate: state() });
                    }
                },
            }
        }
    }
}

#[component]
pub fn SearchResults(query: String) -> Element {
    let info = use_resource(use_reactive!(|query| async move {
        if query.is_empty() {
            return Ok(SearchResponse { crates: vec![] });
        }
        let response = Request::get("https://crates.io/api/v1/crates")
            .query([("q", query.as_str())])
            .build()?
            .send()
            .await?;
        let text = response.json::<SearchResponse>().await?;
        Ok(text) as anyhow::Result<SearchResponse>
    }));

    let info = info.read();
    match &*info {
        None => rsx! {},
        Some(Ok(response)) => rsx! {
            div { class: "flex flex-col gap-2 my-4",
                for c in response.crates.iter() {
                    Card { key: "{c.id}", details: c.clone() }
                }
            }
        },
        Some(Err(error)) => rsx! {
            "Error: "
            {format!("{error:?}")}
        },
    }
}

#[component]
fn GitIcon() -> Element {
    rsx! {
        svg {
            class: "inline",
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 24 24",
            width: "1em",
            height: "1em",
            fill: "currentColor",
            path { d: "M15 4.75a3.25 3.25 0 1 1 6.5 0 3.25 3.25 0 0 1-6.5 0ZM2.5 19.25a3.25 3.25 0 1 1 6.5 0 3.25 3.25 0 0 1-6.5 0Zm0-14.5a3.25 3.25 0 1 1 6.5 0 3.25 3.25 0 0 1-6.5 0ZM5.75 6.5a1.75 1.75 0 1 0-.001-3.501A1.75 1.75 0 0 0 5.75 6.5Zm0 14.5a1.75 1.75 0 1 0-.001-3.501A1.75 1.75 0 0 0 5.75 21Zm12.5-14.5a1.75 1.75 0 1 0-.001-3.501A1.75 1.75 0 0 0 18.25 6.5Z" }
            path { d: "M5.75 16.75A.75.75 0 0 1 5 16V8a.75.75 0 0 1 1.5 0v8a.75.75 0 0 1-.75.75Z" }
            path { d: "M17.5 8.75v-1H19v1a3.75 3.75 0 0 1-3.75 3.75h-7a1.75 1.75 0 0 0-1.75 1.75H5A3.25 3.25 0 0 1 8.25 11h7a2.25 2.25 0 0 0 2.25-2.25Z" }
        }
    }
}

#[component]
fn DocsRsIcon() -> Element {
    rsx! {
        svg {
            class: "inline",
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 512 512",
            width: "1em",
            height: "1em",
            fill: "currentColor",
            path { d: "M488.6 250.2L392 214V105.5c0-15-9.3-28.4-23.4-33.7l-100-37.5c-8.1-3.1-17.1-3.1-25.3 0l-100 37.5c-14.1 5.3-23.4 18.7-23.4 33.7V214l-96.6 36.2C9.3 255.5 0 268.9 0 283.9V394c0 13.6 7.7 26.1 19.9 32.2l100 50c10.1 5.1 22.1 5.1 32.2 0l103.9-52 103.9 52c10.1 5.1 22.1 5.1 32.2 0l100-50c12.2-6.1 19.9-18.6 19.9-32.2V283.9c0-15-9.3-28.4-23.4-33.7zM358 214.8l-85 31.9v-68.2l85-37v73.3zM154 104.1l102-38.2 102 38.2v.6l-102 41.4-102-41.4v-.6zm84 291.1l-85 42.5v-79.1l85-38.8v75.4zm0-112l-102 41.4-102-41.4v-.6l102-38.2 102 38.2v.6zm240 112l-85 42.5v-79.1l85-38.8v75.4zm0-112l-102 41.4-102-41.4v-.6l102-38.2 102 38.2v.6z" }
        }
    }
}

#[component]
fn Card(details: CrateDetail) -> Element {
    let link = Route::Crate {
        krate: details.id.clone(),
    };
    rsx! {
        Link { to: link, class: "card",
            div { class: "header",
                h3 { class: "name", "{details.id}" }
                span { class: "version", "{details.max_version}" }
                span { class: "grow" }
                a {
                    class: "icon",
                    href: "https://docs.rs/{details.id}/{details.max_version}",
                    DocsRsIcon {}
                }
                if let Some(url) = &details.repository {
                    a { class: "icon", href: "{url}", GitIcon {} }
                }
            }
            p { class: "description", "{details.description}" }
        }
    }
}

#[component]
pub fn SummaryLoader(category: Vec<SummaryCategory>) -> Element {
    let info = use_resource(|| SUMMARY_CACHE.fetch_cached());
    let cached = SUMMARY_CACHE.cached();
    if cached.is_none() {
        info.suspend()?;
    }

    let info = info.read();
    let response = match (cached, &*info) {
        (Some(response), _) => response,
        (None, Some(Ok(response))) => response.clone(),
        (None, Some(Err(error))) => {
            return rsx! {
                "Error: "
                {format!("{error:?}")}
            };
        }
        (None, None) => return rsx! {},
    };

    rsx! {
        section { class: "summary",
            for category in category.iter().copied() {
                SummaryColumn { category, crates: response.get(category).clone() }
            }
        }
    }
}

#[component]
fn SummaryColumn(category: SummaryCategory, crates: Vec<CrateDetail>) -> Element {
    if crates.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "column",
            h2 { class: "title", {category.title()} }
            section { class: "results",
                for c in crates.into_iter() {
                    Card { key: "{c.id}", details: c }
                }
            }
        }
    }
}

#[component]
pub fn DefaultSummarySection() -> Element {
    rsx! {
        SuspenseBoundary {
            fallback: |_| rsx! {
                div { class: "p-8",
                    Loading { title: "Loading crates", status: "Fetching the latest crates from crates.io" }
                }
            },
            SummaryLoader {
                category: vec![
                    SummaryCategory::MostRecent,
                    SummaryCategory::MostDownloaded,
                    SummaryCategory::JustUpdated,
                ],
            }
        }
    }
}
