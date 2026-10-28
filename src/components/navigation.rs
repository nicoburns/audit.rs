use crate::{components::Search, data::CrateResponse, *};
use semver::Version;
use std::sync::Arc;

#[component]
fn FileDiffIcon() -> Element {
    // from https://lucide.dev/icons/file-diff
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            height: "1.5ex",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z" }
            path { d: "M9 10h6" }
            path { d: "M12 13V7" }
            path { d: "M9 17h6" }
        }
    }
}

#[component]
fn BoxIcon() -> Element {
    // from https://lucide.dev/icons/box
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            height: "1.5ex",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z" }
            path { d: "m3.3 7 8.7 5 8.7-5" }
            path { d: "M12 22V12" }
        }
    }
}

#[component]
pub fn Navbar(children: Element) -> Element {
    rsx! {
        nav { id: "navbar", class: "", aria_label: "Main",
            div { class: "navbar-items", {children} }
        }
    }
}

#[component]
pub fn NavbarGroup(children: Element) -> Element {
    rsx! {
        div { class: "navbar-group", {children} }
    }
}

#[component]
pub fn NavbarHeading(children: Element) -> Element {
    rsx! {
        div { class: "text-xl font-bold text-nowrap flex flex-row items-center", {children} }
    }
}

#[component]
pub fn NavbarItem(children: Element) -> Element {
    rsx! {
        div { class: "text-lg text-nowrapf flex flex-row items-center", {children} }
    }
}

#[component]
pub fn SimpleNavbar() -> Element {
    rsx! {
        Navbar {
            NavbarGroup {
                NavbarHeading {
                    Link { to: Route::Home {}, class: "flex flex-row items-center",
                        FileDiffIcon {}
                        "diff.rs"
                    }
                }
                NavbarItem {
                    Link { to: Route::About {}, "About" }
                }
            }
        }
    }
}

/// Dropdown, `values` is a list of `(value, label)` pairs.
#[component]
pub fn Select(
    #[props(default)] values: Vec<(String, String)>,
    #[props(default)] selected: Option<String>,
    onchange: Option<EventHandler<String>>,
) -> Element {
    rsx! {
        select {
            class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-1.5 dark:bg-gray-800 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
            onchange: move |event: FormEvent| {
                if let Some(onchange) = onchange {
                    onchange.call(event.value());
                }
            },
            for (key, value) in values.iter() {
                option {
                    key: "{key}",
                    selected: selected.as_ref() == Some(key),
                    value: "{key}",
                    "{value}"
                }
            }
        }
    }
}

#[component]
fn SwitchIcon() -> Element {
    rsx! {
        svg {
            class: "h-4 w-4",
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 24 24",
            width: "24",
            height: "24",
            path {
                fill: "currentColor",
                d: "M7.72 21.78a.75.75 0 0 0 1.06-1.06L5.56 17.5h14.69a.75.75 0 0 0 0-1.5H5.56l3.22-3.22a.75.75 0 1 0-1.06-1.06l-4.5 4.5a.75.75 0 0 0 0 1.06l4.5 4.5Zm8.56-9.5a.75.75 0 1 1-1.06-1.06L18.44 8H3.75a.75.75 0 0 1 0-1.5h14.69l-3.22-3.22a.75.75 0 0 1 1.06-1.06l4.5 4.5a.75.75 0 0 1 0 1.06l-4.5 4.5Z",
            }
        }
    }
}

/// Crate versions selected in the [`ComplexNavbar`], as `((src_name, old), (dst_name, new))`.
pub type VersionSelection = ((String, Version), (String, Version));

#[component]
pub fn ComplexNavbar(
    src_name: String,
    dst_name: String,
    old: Version,
    new: Version,
    src_info: Arc<CrateResponse>,
    dst_info: Arc<CrateResponse>,
    onchange: Option<EventHandler<VersionSelection>>,
) -> Element {
    let prepare_versions = |versions: &[crate::data::VersionInfo]| {
        versions
            .iter()
            .map(|version| {
                let num = version.version.to_string();
                if version.yanked {
                    (num.clone(), format!("{num} (yanked)"))
                } else {
                    (num.clone(), num)
                }
            })
            .collect::<Vec<_>>()
    };
    let mut switched_for = use_signal(|| None::<VersionSelection>);
    let selection = (
        (src_name.clone(), old.clone()),
        (dst_name.clone(), new.clone()),
    );
    let switched = switched_for.read().as_ref() == Some(&selection);

    let (src_name, dst_name, old, new, src_info, dst_info) = if switched {
        (dst_name, src_name, new, old, dst_info, src_info)
    } else {
        (src_name, dst_name, old, new, src_info, dst_info)
    };

    let src_versions = prepare_versions(&src_info.versions);
    let dst_versions = prepare_versions(&dst_info.versions);

    let switch = {
        let new_switched = !switched;
        let versions = if new_switched {
            (
                (dst_name.clone(), new.clone()),
                (src_name.clone(), old.clone()),
            )
        } else {
            (
                (src_name.clone(), old.clone()),
                (dst_name.clone(), new.clone()),
            )
        };
        move |_| {
            switched_for.set(new_switched.then(|| selection.clone()));
            if let Some(onchange) = onchange {
                onchange.call(versions.clone());
            }
        }
    };

    let on_old_change = {
        let src_name = src_name.clone();
        let dst_name = dst_name.clone();
        let new = new.clone();
        move |old: String| {
            let old: Version = old.parse().unwrap();
            if let Some(onchange) = onchange {
                onchange.call(((src_name.clone(), old), (dst_name.clone(), new.clone())));
            }
        }
    };

    let on_new_change = {
        let src_name = src_name.clone();
        let dst_name = dst_name.clone();
        let old = old.clone();
        move |new: String| {
            let new: Version = new.parse().unwrap();
            if let Some(onchange) = onchange {
                onchange.call(((src_name.clone(), old.clone()), (dst_name.clone(), new)));
            }
        }
    };

    rsx! {
        Navbar {
            NavbarHeading {
                Link { to: Route::Home {}, class: "flex flex-row items-center",
                    FileDiffIcon {}
                    span { "diff.rs" }
                }
            }
            div { class: "navbar-group lg:order-last grow min-w-[370px]", Search {} }
            div { class: "navbar-group grow",
                div { class: "navbar-group flex-nowrap",
                    NavbarItem {
                        a {
                            href: "https://crates.io/crates/{src_name}",
                            class: "flex flex-row items-center",
                            BoxIcon {}
                        }
                        "{src_name}"
                    }
                    NavbarItem {
                        Select {
                            values: src_versions,
                            selected: old.to_string(),
                            onchange: on_old_change,
                        }
                    }
                }
                NavbarItem {
                    span {
                        class: "cursor-pointer hover:rotate-180 transition delay-150 duration-300 ease-in-out",
                        onclick: switch,
                        SwitchIcon {}
                    }
                }
                div { class: "navbar-group flex-nowrap",
                    NavbarItem {
                        a {
                            href: "https://crates.io/crates/{dst_name}",
                            class: "flex flex-row items-center",
                            BoxIcon {}
                        }
                        "{dst_name}"
                    }
                    NavbarItem {
                        Select {
                            values: dst_versions,
                            selected: new.to_string(),
                            onchange: on_new_change,
                        }
                    }
                }
            }
        }
    }
}
