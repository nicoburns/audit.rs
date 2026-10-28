use crate::{
    data::{ChunkInfo, FileDiff, VersionDiff},
    syntax::{highlight_changes, infer_syntax_for_file, syntect_style_to_css},
};
use bytes::Bytes;
use camino::Utf8PathBuf;
use dioxus::{logger::tracing::info, prelude::*};
use similar::ChangeTag;
use std::rc::Rc;
use syntect::highlighting::Style;

/// Contains information about contiguous changes
#[derive(PartialEq, Clone)]
struct DiffGroupInfo {
    /// The actual changes
    group: Vec<(ChangeTag, Vec<(Style, bytes::Bytes)>)>,
    /// What range of lines the group covers (used as a list key)
    range: ChunkInfo,
    /// Whether the group contains an actual diff (and therefore shows some context)
    in_context: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum DiffStyle {
    #[default]
    Unified,
    Split,
}

#[component]
fn FileIcon() -> Element {
    // from https://www.svgrepo.com/svg/491619/doc
    rsx! {
        svg {
            class: "fill-gray-500 w-4",
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
pub fn DiffView(path: Utf8PathBuf, diff: Rc<VersionDiff>) -> Element {
    let empty = FileDiff::default();
    let file_diff = diff.files.get(&path).unwrap_or(&empty);
    let summary = diff.summary.get(&path).unwrap_or(&(0, 0));
    let is_identical_version = diff.left.version == diff.right.version;

    // Determine which syntax should be used for this file. It will be based
    // first on the file's name, then the file's extension, then the first line.
    let syntax = infer_syntax_for_file(
        path.as_str(),
        file_diff
            .changes
            .iter()
            .find(|(tag, _)| *tag != ChangeTag::Delete)
            .and_then(|(_, line)| std::str::from_utf8(line).ok()),
    );
    info!("Highlighting {} as {}", syntax.name, path);

    // Apply highlighting to every change in the file.
    let mut changes = highlight_changes(syntax, &file_diff.changes).into_iter();
    let ranges = file_diff.context_ranges.iter();

    // Group contiguous lines by whether they contain an actual diff +/- some context buffer.
    let mut cursor = 0;
    let mut stack: Vec<DiffGroupInfo> = vec![];

    for next_range in ranges {
        // out of context lines
        if next_range.start() != 0 {
            stack.push(DiffGroupInfo {
                group: changes.by_ref().take(next_range.start() - cursor).collect(),
                range: ChunkInfo {
                    range: cursor..next_range.start(),
                    left_start: (next_range.left_start + cursor).saturating_sub(next_range.start()),
                    right_start: (next_range.right_start + cursor)
                        .saturating_sub(next_range.start()),
                },
                in_context: false,
            });
        }
        // in context lines
        stack.push(DiffGroupInfo {
            group: changes
                .by_ref()
                .take(next_range.end() - next_range.start())
                .collect(),
            range: next_range.clone(),
            in_context: true,
        });
        cursor = next_range.end();
    }
    if changes.len() > 0 {
        // Trailing unchanged lines at the end of a file
        stack.push(DiffGroupInfo {
            group: changes.by_ref().collect(),
            range: ChunkInfo {
                range: cursor..file_diff.changes.len(),
                left_start: (cursor).saturating_sub(file_diff.summary.added as usize),
                right_start: (cursor).saturating_sub(file_diff.summary.removed as usize),
            },

            // When comparing a version of the crate to itself, this group will
            // always contain the full text of the file. Don't collapse it.
            in_context: is_identical_version,
        });
    }

    rsx! {
        div { class: "diff-view",
            div { class: "header",
                FileIcon {}
                span { class: "filename", {path.file_name().unwrap_or("")} }
            }
            div { class: "content",
                if summary == &(0, 0) {
                    FileDisplayView { stack }
                } else {
                    UnifiedDiffView { stack }
                }
            }
        }
    }
}

/// Pairs every group with the overall index of its first line.
fn with_overall_index(stack: Vec<DiffGroupInfo>) -> impl Iterator<Item = (usize, DiffGroupInfo)> {
    stack.into_iter().scan(0, |overall_index, info| {
        let start = *overall_index;
        *overall_index += info.group.len();
        Some((start, info))
    })
}

#[component]
fn UnifiedDiffView(stack: Vec<DiffGroupInfo>) -> Element {
    rsx! {
        div { class: "overflow-x-scroll bg-white",
            div { class: "unified",
                for (overall_index , DiffGroupInfo { group , range , in_context }) in with_overall_index(stack) {
                    DiffLineGroup {
                        key: "{range.range:?}",
                        group,
                        in_context,
                        group_start_index: (overall_index, range.left_start, range.right_start),
                    }
                }
            }
        }
    }
}

#[component]
fn FileDisplayView(stack: Vec<DiffGroupInfo>) -> Element {
    rsx! {
        div { class: "overflow-x-scroll bg-white",
            div { class: "unified",
                for (overall_index , DiffGroupInfo { group , range , in_context: _ }) in with_overall_index(stack) {
                    FileView {
                        key: "{range:?}",
                        group: group.into_iter().map(|(_, line)| line).collect::<Vec<_>>(),
                        group_start_index: overall_index,
                    }
                }
            }
        }
    }
}

#[component]
fn SplitDiffView(stack: Vec<DiffGroupInfo>) -> Element {
    rsx! {
        div { class: "p-2 overflow-x-scroll bg-white",
            pre { class: "bg-white",
                for (overall_index , DiffGroupInfo { group , range , in_context }) in with_overall_index(stack) {
                    DiffLineGroup {
                        key: "{range:?}",
                        group,
                        in_context,
                        group_start_index: (overall_index, range.left_start, range.right_start),
                    }
                }
            }
        }
    }
}

#[component]
fn ExpandIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 16 16",
            fill: "currentColor",
            height: "1em",
            width: "1em",
            class: "inline",
            path { d: "m8.177.677 2.896 2.896a.25.25 0 0 1-.177.427H8.75v1.25a.75.75 0 0 1-1.5 0V4H5.104a.25.25 0 0 1-.177-.427L7.823.677a.25.25 0 0 1 .354 0ZM7.25 10.75a.75.75 0 0 1 1.5 0V12h2.146a.25.25 0 0 1 .177.427l-2.896 2.896a.25.25 0 0 1-.354 0l-2.896-2.896A.25.25 0 0 1 5.104 12H7.25v-1.25Zm-5-2a.75.75 0 0 0 0-1.5h-.5a.75.75 0 0 0 0 1.5h.5ZM6 8a.75.75 0 0 1-.75.75h-.5a.75.75 0 0 1 0-1.5h.5A.75.75 0 0 1 6 8Zm2.25.75a.75.75 0 0 0 0-1.5h-.5a.75.75 0 0 0 0 1.5h.5ZM12 8a.75.75 0 0 1-.75.75h-.5a.75.75 0 0 1 0-1.5h.5A.75.75 0 0 1 12 8Zm2.25.75a.75.75 0 0 0 0-1.5h-.5a.75.75 0 0 0 0 1.5h.5Z" }
        }
    }
}

#[component]
fn DiffLineGroup(
    group: Vec<(ChangeTag, Vec<(Style, bytes::Bytes)>)>,
    in_context: bool,
    group_start_index: (usize, usize, usize),
) -> Element {
    let mut folded = use_signal(|| !in_context);

    // go from 0-indexed to 1-indexed
    let start_index = (
        group_start_index.0 + 1,
        group_start_index.1 + 1,
        group_start_index.2 + 1,
    );

    // use the fact that folded sections never contain changes
    let end_index = (
        start_index.0 + group.len() - 1,
        start_index.1 + group.len() - 1,
        start_index.2 + group.len() - 1,
    );

    if folded() {
        let info = if start_index.1 == start_index.2 {
            format!("Show lines {:?} to {:?}", start_index.1, end_index.1)
        } else {
            format!(
                "Show lines {:?} to {:?}",
                (start_index.1, start_index.2),
                (end_index.1, end_index.2)
            )
        };
        rsx! {
            div { class: "expand",
                button { class: "button", onclick: move |_| folded.toggle(), ExpandIcon {} }
                button { class: "info", onclick: move |_| folded.toggle(), "{info}" }
            }
        }
    } else {
        let (mut left_idx, mut right_idx) = (start_index.1, start_index.2);
        let lines = group.into_iter().map(|(tag, change)| {
            let (sign, class, left, right) = match tag {
                ChangeTag::Delete => ("-", "line deletion", Some(left_idx), None),
                ChangeTag::Insert => ("+", "line insertion", None, Some(right_idx)),
                ChangeTag::Equal => (" ", "line unchanged", Some(left_idx), Some(right_idx)),
            };
            (left_idx, right_idx) = match tag {
                ChangeTag::Delete => (left_idx + 1, right_idx),
                ChangeTag::Insert => (left_idx, right_idx + 1),
                ChangeTag::Equal => (left_idx + 1, right_idx + 1),
            };

            rsx! {
                div { class,
                    a { id: left.map(|i| format!("L{i}")), class: "line-number",
                        if let Some(index) = left {
                            "{index}"
                        }
                    }
                    a { id: right.map(|i| format!("R{i}")), class: "line-number",
                        if let Some(index) = right {
                            "{index}"
                        }
                    }
                    div { class: "change-icon", "{sign}" }
                    div { class: "code-line",
                        CodeLine { stack: change }
                    }
                }
            }
        });
        rsx! {
            {lines}
        }
    }
}

#[component]
fn FileView(group: Vec<Vec<(Style, bytes::Bytes)>>, group_start_index: usize) -> Element {
    rsx! {
        for (index , change) in group.into_iter().enumerate() {
            div { class: "line unchanged",
                div { class: "line-number file-view", "{index + 1 + group_start_index}" }
                div { class: "code-line",
                    CodeLine { stack: change }
                }
            }
        }
    }
}

#[component]
fn CodeLine(stack: Vec<(Style, Bytes)>) -> Element {
    rsx! {
        for (style , text) in stack.iter() {
            span { style: syntect_style_to_css(style), {String::from_utf8_lossy(&text[..]).into_owned()} }
        }
    }
}
