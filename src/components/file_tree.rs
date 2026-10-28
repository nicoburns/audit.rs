use crate::{
    components::SearchGlass,
    data::{Changes, Entry, Item, VersionDiff},
    Route, VersionId,
};
use camino::Utf8PathBuf;
use dioxus::prelude::*;
use std::rc::Rc;

#[derive(PartialEq, Clone, Debug)]
struct Context {
    old_krate: String,
    old_version: VersionId,
    new_krate: String,
    new_version: VersionId,
}

impl Context {
    fn file_route(&self, path: Utf8PathBuf) -> Route {
        Route::File {
            old_krate: self.old_krate.clone(),
            old_version: self.old_version.clone(),
            new_krate: self.new_krate.clone(),
            new_version: self.new_version.clone(),
            path: path.into(),
        }
        .simplify()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
enum ChangeFilter {
    #[default]
    All,
    Changed,
}

impl ChangeFilter {
    fn is_all(&self) -> bool {
        matches!(self, Self::All)
    }

    fn is_changed(&self) -> bool {
        matches!(self, Self::Changed)
    }

    fn matches(&self, changes: Changes) -> bool {
        match self {
            Self::All => true,
            Self::Changed => changes != Changes::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
enum SearchFilter {
    #[default]
    All,
    Filter(String),
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for SearchFilter {
    fn to_string(&self) -> String {
        match self {
            SearchFilter::All => "".to_string(),
            SearchFilter::Filter(s) => s.to_string(),
        }
    }
}

impl SearchFilter {
    fn matches(&self, name: &str) -> bool {
        match self {
            Self::All => true,
            Self::Filter(s) => name.to_lowercase().contains(&s.to_lowercase()),
        }
    }

    fn match_entry(&self, entry: &Entry) -> bool {
        match &entry.item {
            Item::File => self.matches(&entry.name),
            Item::Dir(e) => self.matches(&entry.name) || e.iter().any(|(_, e)| self.match_entry(e)),
        }
    }
}

#[component]
fn FolderIcon() -> Element {
    // from https://www.svgrepo.com/svg/491619/doc
    rsx! {
        svg {
            class: "fill-blue-300 dark:fill-gray-600",
            view_box: "0 0 24 24",
            fill: "currentColor",
            xmlns: "http://www.w3.org/2000/svg",
            path { d: "M4 2C3.20435 2 2.44129 2.31607 1.87868 2.87868C1.31607 3.44129 1 4.20435 1 5V19C1 19.7957 1.31607 20.5587 1.87868 21.1213C2.44129 21.6839 3.20435 22 4 22H20C20.7957 22 21.5587 21.6839 22.1213 21.1213C22.6839 20.5587 23 19.7957 23 19V8C23 7.20435 22.6839 6.44129 22.1213 5.87868C21.5587 5.31607 20.7957 5 20 5H11.5352L10.1289 2.8906C9.75799 2.3342 9.13352 2 8.46482 2H4Z" }
        }
    }
}

#[component]
fn FileIcon() -> Element {
    // from https://www.svgrepo.com/svg/491619/doc
    rsx! {
        svg {
            class: "fill-gray-500",
            view_box: "0 0 24 24",
            fill: "currentColor",
            xmlns: "http://www.w3.org/2000/svg",
            path {
                fill_rule: "evenodd",
                clip_rule: "evenodd",
                d: "M6 1C4.34314 1 3 2.34315 3 4V20C3 21.6569 4.34315 23 6 23H19C20.6569 23 22 21.6569 22 20V10C22 9.73478 21.8946 9.48043 21.7071 9.29289L13.7071 1.29292C13.6114 1.19722 13.4983 1.1229 13.3753 1.07308C13.2572 1.02527 13.1299 1 13 1H6ZM12 3H6C5.44771 3 5 3.44771 5 4V20C5 20.5523 5.44772 21 6 21H19C19.5523 21 20 20.5523 20 20V11H13C12.4477 11 12 10.5523 12 10V3ZM18.5858 9.00003L14 4.41424V9.00003H18.5858Z",
            }
        }
    }
}

#[component]
fn ExpandIcon(is_expanded: bool) -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "currentColor",
            xmlns: "http://www.w3.org/2000/svg",
            class: is_expanded.then_some("rotate-90"),
            path {
                fill_rule: "evenodd",
                clip_rule: "evenodd",
                d: "M8.79289 6.29289C9.18342 5.90237 9.81658 5.90237 10.2071 6.29289L15.2071 11.2929C15.5976 11.6834 15.5976 12.3166 15.2071 12.7071L10.2071 17.7071C9.81658 18.0976 9.18342 18.0976 8.79289 17.7071C8.40237 17.3166 8.40237 16.6834 8.79289 16.2929L13.0858 12L8.79289 7.70711C8.40237 7.31658 8.40237 6.68342 8.79289 6.29289Z",
            }
        }
    }
}

#[component]
fn FileEntry(
    context: Rc<Context>,
    entry: Rc<Entry>,
    #[props(default)] active: Rc<Utf8PathBuf>,
    #[props(default)] prefix: Rc<Utf8PathBuf>,
    #[props(default)] change_filter: ChangeFilter,
    #[props(default)] search_filter: SearchFilter,
) -> Element {
    let path = {
        let mut path = (*prefix).clone();
        path.push(&entry.name);
        path
    };

    let mut expanded = use_signal(|| false);
    let current = path == *active;

    let route = context.file_route(path.clone());

    let toggle_expand = move |event: MouseEvent| {
        event.prevent_default();
        expanded.toggle();
    };

    if !search_filter.match_entry(&entry) {
        return rsx! {};
    }

    rsx! {
        Link {
            to: route,
            class: if current { "file-entry active" } else { "file-entry" },
            button {
                class: if expanded() { "toggle active" } else { "toggle" },
                onclick: toggle_expand,
                if entry.item.is_dir() {
                    ExpandIcon { is_expanded: expanded() }
                }
            }
            div { class: "icon",
                if entry.item.is_dir() {
                    FolderIcon {}
                } else {
                    FileIcon {}
                }
            }
            div { class: "name", "{entry.name}" }
            div { class: "tags",
                if entry.changes.added > 0 {
                    span { class: "tag added", "+{entry.changes.added}" }
                }
                if entry.changes.removed > 0 {
                    span { class: "tag removed", "-{entry.changes.removed}" }
                }
            }
        }
        if entry.item.is_dir() && expanded() {
            SubTree {
                entry: entry.clone(),
                context: context.clone(),
                prefix: prefix.clone(),
                active: active.clone(),
                change_filter,
                search_filter: search_filter.clone(),
            }
        }
    }
}

#[component]
fn SubTree(
    context: Rc<Context>,
    entry: Rc<Entry>,
    #[props(default)] active: Rc<Utf8PathBuf>,
    #[props(default)] prefix: Rc<Utf8PathBuf>,
    #[props(default)] change_filter: ChangeFilter,
    #[props(default)] search_filter: SearchFilter,
) -> Element {
    debug_assert!(entry.item.is_dir());

    // build new prefix
    let mut new_prefix = (*prefix).clone();
    new_prefix.push(&entry.name);
    let prefix = Rc::new(new_prefix);

    let entries = match &entry.item {
        Item::File => unreachable!(),
        Item::Dir(entries) => entries,
    };

    rsx! {
        div { class: "file-subtree",
            for (key , entry) in entries
                .iter()
                .filter(|(_, entry)| change_filter.matches(entry.changes))
                .filter(|(_, entry)| search_filter.matches(&entry.name) || entry.item.is_dir())
            {
                FileEntry {
                    key: "{key}",
                    context: context.clone(),
                    entry: entry.clone(),
                    prefix: prefix.clone(),
                    active: active.clone(),
                    change_filter,
                    search_filter: search_filter.clone(),
                }
            }
        }
    }
}

#[component]
pub fn FileTree(diff: Rc<VersionDiff>, path: Utf8PathBuf) -> Element {
    let entries = match diff.tree.item.clone() {
        Item::File => Default::default(),
        Item::Dir(entries) => entries,
    };

    let mut change_filter = use_signal(|| ChangeFilter::All);
    let change_filter_set = move |filter: ChangeFilter| {
        move |event: MouseEvent| {
            change_filter.set(filter);
            event.prevent_default();
        }
    };

    let search_filter = use_signal(|| SearchFilter::All);
    let prefix = Rc::new(Utf8PathBuf::default());
    let active = Rc::new(path);

    let context = Rc::new(Context {
        old_krate: diff.left.version.krate.clone(),
        old_version: diff.left.version.version.clone().into(),
        new_krate: diff.right.version.krate.clone(),
        new_version: diff.right.version.version.clone().into(),
    });

    rsx! {
        div { class: "file-tree",
            div { class: "header",
                FileSearch { filter: search_filter }
                div { class: "button-group", role: "group",
                    button {
                        r#type: "button",
                        class: if change_filter.read().is_all() { "first active" } else { "first" },
                        onclick: change_filter_set(ChangeFilter::All),
                        "all"
                    }
                    button {
                        r#type: "button",
                        class: if change_filter.read().is_changed() { "last active" } else { "last" },
                        onclick: change_filter_set(ChangeFilter::Changed),
                        "changed"
                    }
                }
            }
            for (key , entry) in entries
                .into_iter()
                .filter(|(_, entry)| change_filter.read().matches(entry.changes))
                .filter(|(_, entry)| search_filter.read().matches(&entry.name) || entry.item.is_dir())
            {
                FileEntry {
                    key: "{key}",
                    entry,
                    prefix: prefix.clone(),
                    active: active.clone(),
                    context: context.clone(),
                    change_filter: change_filter(),
                    search_filter: search_filter(),
                }
            }
        }
    }
}

#[component]
fn FileSearch(filter: Signal<SearchFilter>) -> Element {
    rsx! {
        div { class: "relative w-full",
            div { class: "absolute inset-y-0 start-0 flex items-center ps-3 pointer-events-none",
                SearchGlass {}
            }
            input {
                r#type: "search",
                class: "block w-full p-1 ps-10 text-gray-900 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-800 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
                placeholder: "Filter...",
                value: filter.read().to_string(),
                oninput: move |event: FormEvent| filter.set(SearchFilter::Filter(event.value())),
            }
        }
    }
}
