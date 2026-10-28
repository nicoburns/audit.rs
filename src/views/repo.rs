use crate::{
    cache::*,
    components::*,
    data::{CrateResponse, CrateSource, RepositoryInfo, VersionDiff, VersionInfo},
    version::VersionId,
};
use camino::Utf8PathBuf;
use dioxus::prelude::*;
use std::{rc::Rc, sync::Arc};

#[component]
pub fn RepoFileView(krate: String, version: VersionId, path: Utf8PathBuf) -> Element {
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
            CrateFetcher { krate, version, path }
        }
    }
}

#[component]
fn CrateFetcher(krate: String, version: VersionId, path: Utf8PathBuf) -> Element {
    let info = use_resource(use_reactive!(|krate| async move {
        CRATE_RESPONSE_CACHE.fetch_cached(&krate).await
    }))
    .suspend()?;

    let info = match &*info.read() {
        Ok(info) => info.clone(),
        Err(error) => {
            return rsx! {
                SimpleNavbar {}
                Content {
                    Center {
                        Error { title: "Loading crate", status: "Error: {error}" }
                    }
                }
            };
        }
    };

    rsx! {
        VersionResolver { info, version, path }
    }
}

#[component]
fn VersionResolver(info: Arc<CrateResponse>, version: VersionId, path: Utf8PathBuf) -> Element {
    let Some(version) = info.version(version) else {
        return rsx! { "version not found" };
    };

    rsx! {
        CrateSourceFetcher { info: info.clone(), version: version.clone(), path }
    }
}

#[component]
fn CrateSourceFetcher(
    info: Arc<CrateResponse>,
    version: VersionInfo,
    path: Utf8PathBuf,
) -> Element {
    rsx! {
        SuspenseBoundary { fallback: |_| rsx! { "Loading" },
            CrateSourceFetcherInner { info, version, path }
        }
    }
}

#[component]
fn CrateSourceFetcherInner(
    info: Arc<CrateResponse>,
    version: VersionInfo,
    path: Utf8PathBuf,
) -> Element {
    let source = use_resource(use_reactive!(|version| async move {
        CRATE_SOURCE_CACHE.fetch_cached(&version).await
    }))
    .suspend()?;

    let source = match &*source.read() {
        Ok(source) => source.clone(),
        Err(_error) => {
            return rsx! { "Error fetching source" };
        }
    };

    rsx! {
        RepoSourceFetcher { info, source, path }
    }
}

#[component]
fn RepoSourceFetcher(
    info: Arc<CrateResponse>,
    source: Arc<CrateSource>,
    path: Utf8PathBuf,
) -> Element {
    let Some(repository) = info.krate.repository.clone() else {
        return rsx! { "No repository set in crate metadata" };
    };

    let vcs_info = match source.cargo_vcs_info() {
        Ok(info) => info,
        Err(error) => {
            return rsx! {
                "error: "
                {error.to_string()}
            };
        }
    };

    let repository = RepositoryInfo {
        repository,
        vcs_info,
    };

    rsx! {
        SuspenseBoundary { fallback: |_| rsx! { "Loading repository archive" },
            RepoSourceFetcherInner {
                repository,
                source,
                path,
            }
        }
    }
}

#[component]
fn RepoSourceFetcherInner(
    source: Arc<CrateSource>,
    repository: RepositoryInfo,
    path: Utf8PathBuf,
) -> Element {
    let repo_source = use_resource(use_reactive!(|repository| async move {
        repository.fetch().await.map(Arc::new)
    }))
    .suspend()?;

    let repo_source = match &*repo_source.read() {
        Ok(repo_source) => repo_source.clone(),
        Err(error) => {
            return rsx! {
                {format!("Error repo: {error}")}
            }
        }
    };

    let diff = VersionDiff::new(repo_source, source);
    let diff = Rc::new(diff);

    rsx! {
        Content {
            main { class: "flex flex-col md:flex-row gap-2 lg:gap-4 p-2",
                nav {
                    id: "files",
                    class: "md:w-72 lg:w-84 xl:w-96",
                    aria_label: "Files",
                    FileTree { diff: diff.clone(), path: path.clone() }
                }
                div { id: "diff-view", class: "flex-1",
                    DiffView { diff, path }
                }
            }
        }
    }
}
