use crate::{cache::*, components::*, data::*, version::VersionId, Route};
use camino::Utf8PathBuf;
use dioxus::prelude::*;
use std::{rc::Rc, sync::Arc};

/// Show diff of a file change between two crate versions.
#[component]
pub fn Diff(
    src_name: String,
    dst_name: String,
    old: VersionId,
    new: VersionId,
    path: Option<Utf8PathBuf>,
) -> Element {
    rsx! {
        SuspenseBoundary {
            fallback: |_| rsx! {
                SimpleNavbar {}
                Content {
                    Center {
                        Loading { title: "Loading crate", status: "Loading crate metadata" }
                    }
                }
            },
            CrateFetcher {
                src_name,
                dst_name,
                old,
                new,
                path,
            }
        }
    }
}

#[component]
fn CrateFetcher(
    src_name: String,
    dst_name: String,
    old: VersionId,
    new: VersionId,
    path: Option<Utf8PathBuf>,
) -> Element {
    let info = use_resource(use_reactive!(|src_name, dst_name| async move {
        (
            CRATE_RESPONSE_CACHE.fetch_cached(&src_name).await,
            CRATE_RESPONSE_CACHE.fetch_cached(&dst_name).await,
        )
    }))
    .suspend()?;

    let errors = match &*info.read() {
        (Ok(src_info), Ok(dst_info)) => {
            return rsx! {
                VersionResolver {
                    src_info: src_info.clone(),
                    dst_info: dst_info.clone(),
                    old,
                    new,
                    path,
                }
            }
        }
        (Err(error), Ok(_)) => vec![(&src_name, error.to_string())],
        (Ok(_), Err(error)) => vec![(&dst_name, error.to_string())],
        (Err(src_error), Err(dst_error)) => {
            vec![
                (&src_name, src_error.to_string()),
                (&dst_name, dst_error.to_string()),
            ]
        }
    };
    let errors = errors
        .iter()
        .map(|(name, error)| format!("{name} with {error}"))
        .collect::<Vec<_>>()
        .join(" and ");
    rsx! {
        SimpleNavbar {}
        Content {
            Center {
                Error { title: "Loading crate", status: "Error: {errors}" }
            }
        }
    }
}

#[component]
fn VersionResolver(
    src_info: Arc<CrateResponse>,
    dst_info: Arc<CrateResponse>,
    old: VersionId,
    new: VersionId,
    path: Option<Utf8PathBuf>,
) -> Element {
    // find krate version info
    let old_info = src_info.version(old.clone());
    let new_info = dst_info.version(new.clone());
    let errors = match (old_info, new_info) {
        (Some(old_info), Some(new_info)) => {
            return rsx! {
                SourceFetcher {
                    src_info: src_info.clone(),
                    dst_info: dst_info.clone(),
                    old: old_info.clone(),
                    new: new_info.clone(),
                    path,
                }
            }
        }
        // get invalid versions from props
        (None, Some(_)) => vec![(&src_info, &old)],
        (Some(_), None) => vec![(&dst_info, &new)],
        (None, None) => vec![(&src_info, &old), (&dst_info, &new)],
    };
    let errors = errors
        .iter()
        .map(|(info, version)| format!("Error: version {version} of {} not found", info.krate.id))
        .collect::<Vec<_>>()
        .join(" and ");
    rsx! {
        SimpleNavbar {}
        Content {
            Center {
                Error { title: "Resolving version", status: errors }
            }
        }
    }
}

#[component]
fn SourceFetcher(
    src_info: Arc<CrateResponse>,
    dst_info: Arc<CrateResponse>,
    old: VersionInfo,
    new: VersionInfo,
    path: Option<Utf8PathBuf>,
) -> Element {
    let fallback = {
        let src_info = src_info.clone();
        let dst_info = dst_info.clone();
        let old = old.version.clone();
        let new = new.version.clone();
        move |_| {
            rsx! {
                ComplexNavbar {
                    src_name: src_info.krate.id.clone(),
                    dst_name: dst_info.krate.id.clone(),
                    old: old.clone(),
                    new: new.clone(),
                    src_info: src_info.clone(),
                    dst_info: dst_info.clone(),
                }
                Center {
                    Loading { title: "Loading crate", status: "Loading crate source" }
                }
            }
        }
    };
    rsx! {
        SuspenseBoundary { fallback,
            SourceFetcherInner {
                src_info,
                dst_info,
                old,
                new,
                path,
            }
        }
    }
}

/// Navigates to the given route once rendered.
#[component]
fn Redirect(to: Route) -> Element {
    let navigator = use_navigator();
    use_effect(use_reactive!(|to| {
        navigator.replace(to);
    }));
    rsx! {}
}

/// Builds the handler that navigates to the selected crate versions, keeping the current file.
fn navigate_to_versions(path: Utf8PathBuf) -> impl FnMut(VersionSelection) {
    let navigator = navigator();
    move |((src_name, old), (dst_name, new)): VersionSelection| {
        navigator.push(
            Route::File {
                old_krate: src_name,
                new_krate: dst_name,
                old_version: old.into(),
                new_version: new.into(),
                path: path.clone().into(),
            }
            .simplify(),
        );
    }
}

#[component]
fn SourceFetcherInner(
    src_info: Arc<CrateResponse>,
    dst_info: Arc<CrateResponse>,
    old: VersionInfo,
    new: VersionInfo,
    path: Option<Utf8PathBuf>,
) -> Element {
    // fetch old version source
    let old_source = use_resource(use_reactive!(|old| async move {
        CRATE_SOURCE_CACHE.fetch_cached(&old).await
    }));

    // fetch new version source
    let new_source = use_resource(use_reactive!(|new| async move {
        CRATE_SOURCE_CACHE.fetch_cached(&new).await
    }));

    let old_source = old_source.suspend()?;
    let new_source = new_source.suspend()?;

    let (old_source, new_source) = match (&*old_source.read(), &*new_source.read()) {
        (Ok(old_source), Ok(new_source)) => (old_source.clone(), new_source.clone()),
        (Err(error), _) | (_, Err(error)) => {
            return rsx! {
                ComplexNavbar {
                    src_name: src_info.krate.id.clone(),
                    dst_name: dst_info.krate.id.clone(),
                    old: old.version.clone(),
                    new: new.version.clone(),
                    src_info: src_info.clone(),
                    dst_info: dst_info.clone(),
                    onchange: navigate_to_versions(path.clone().unwrap_or_default()),
                }
                Content {
                    Center {
                        Error { title: "Loading crate", status: "Error: {error}" }
                    }
                }
            };
        }
    };

    dbg!(&path);
    let path = match path {
        None => {
            return rsx! {
                Redirect {
                    to: Route::File {
                        old_krate: src_info.krate.id.clone(),
                        new_krate: dst_info.krate.id.clone(),
                        old_version: old.version.clone().into(),
                        new_version: new.version.clone().into(),
                        path: "Cargo.toml".into(),
                    }
                        .simplify(),
                }
            }
        }
        Some(path) => path,
    };

    rsx! {
        div { class: "",
            SourceView {
                src_info,
                dst_info,
                old: old_source,
                new: new_source,
                path,
            }
        }
    }
}

#[component]
pub fn SourceView(
    src_info: Arc<CrateResponse>,
    dst_info: Arc<CrateResponse>,
    old: Arc<CrateSource>,
    new: Arc<CrateSource>,
    path: Utf8PathBuf,
) -> Element {
    let diff = use_memo(use_reactive!(|old, new| Rc::new(VersionDiff::new(
        old, new
    ))));
    rsx! {
        ComplexNavbar {
            src_name: src_info.krate.id.clone(),
            dst_name: dst_info.krate.id.clone(),
            old: old.version.version.clone(),
            new: new.version.version.clone(),
            src_info: src_info.clone(),
            dst_info: dst_info.clone(),
            onchange: navigate_to_versions(path.clone()),
        }
        Content {
            main { class: "flex flex-col md:flex-row gap-2 lg:gap-4 p-2",
                nav {
                    id: "files",
                    class: "md:w-72 lg:w-84 xl:w-96",
                    aria_label: "Files",
                    FileTree { diff: diff(), path: path.clone() }
                }
                div { id: "diff-view", class: "flex-1",
                    DiffView { diff: diff(), path }
                }
            }
        }
    }
}
