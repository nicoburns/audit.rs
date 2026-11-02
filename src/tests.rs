use crate::data::*;
use anyhow::Result;
use serde_json::from_reader;
use std::fs::File;

fn parse_canned_response(name: &str) -> Result<CrateResponse> {
    let response = File::open(format!("data/{name}.json"))?;
    let mut response: CrateResponse = from_reader(response)?;
    response.sort_versions();
    Ok(response)
}

fn parse_canned_source(version: &VersionInfo) -> Result<CrateSource> {
    let data = std::fs::read(format!("data/{}-{}.crate", version.krate, version.version))?;
    let source = CrateSource::new(version.clone(), &data[..])?;
    Ok(source)
}

#[test]
fn test_crate_response_decode_serde() {
    let response = parse_canned_response("serde").unwrap();
    assert_eq!(response.krate.id, "serde");
}

#[test]
fn test_crate_response_decode_axum() {
    let response = parse_canned_response("axum").unwrap();
    assert_eq!(response.krate.id, "axum");
}

#[test]
fn test_crate_response_decode_reqwest() {
    let response = parse_canned_response("reqwest").unwrap();
    assert_eq!(response.krate.id, "reqwest");
}

#[test]
fn test_crate_response_decode_log() {
    let response = parse_canned_response("log").unwrap();
    assert_eq!(response.krate.id, "log");
}

#[test]
fn crate_response_versions_sorted_by_semver() {
    use crate::version::{VersionId, VersionNamed};

    let response = parse_canned_response("servo-malloc-size-of").unwrap();
    let versions: Vec<String> = response
        .versions
        .iter()
        .map(|v| v.version.to_string())
        .collect();
    assert_eq!(
        versions,
        [
            "0.7.0",
            "0.6.0",
            "0.5.0",
            "0.4.0",
            "0.3.0",
            "0.2.0",
            "0.1.4",
            "0.1.3",
            "0.1.2",
            "0.1.0",
            "0.1.0-rc2",
            "0.1.0-rc1",
        ]
    );
    let named = |name| {
        response
            .version(VersionId::Named(name))
            .unwrap()
            .version
            .to_string()
    };
    assert_eq!(named(VersionNamed::Latest), "0.7.0");
    assert_eq!(named(VersionNamed::Previous), "0.6.0");
}

#[test]
fn crate_response_sorts_prereleases_like_cargo() {
    // Precedence order from the SemVer spec, which Cargo follows.
    let expected = [
        "1.0.0",
        "1.0.0-rc.1",
        "1.0.0-beta.11",
        "1.0.0-beta.2",
        "1.0.0-beta",
        "1.0.0-alpha.beta",
        "1.0.0-alpha.1",
        "1.0.0-alpha",
        "0.9.0",
    ];
    let mut shuffled = expected.to_vec();
    shuffled.reverse();
    shuffled.swap(1, 6);
    let versions: Vec<_> = shuffled
        .iter()
        .map(|num| {
            serde_json::json!({
                "checksum": "00",
                "crate": "test",
                "dl_path": "",
                "yanked": false,
                "num": num,
            })
        })
        .collect();
    let mut response: CrateResponse = serde_json::from_value(serde_json::json!({
        "crate": {
            "id": "test",
            "max_version": "1.0.0",
            "max_stable_version": "1.0.0",
            "newest_version": "1.0.0",
            "description": "",
            "downloads": 0,
            "recent_downloads": null,
            "exact_match": true,
            "homepage": null,
            "repository": null,
            "documentation": null,
        },
        "versions": versions,
    }))
    .unwrap();
    response.sort_versions();
    let sorted: Vec<String> = response
        .versions
        .iter()
        .map(|v| v.version.to_string())
        .collect();
    assert_eq!(sorted, expected);
}

#[test]
fn can_parse_crate_source_log_0_4_15() {
    let log = parse_canned_response("log").unwrap();
    let version = log.version("0.4.15".parse().unwrap()).unwrap();
    let _ = parse_canned_source(version).unwrap();
}

#[test]
fn can_parse_crate_source_log_0_4_16() {
    let log = parse_canned_response("log").unwrap();
    let version = log.version("0.4.16".parse().unwrap()).unwrap();
    let _ = parse_canned_source(version).unwrap();
}

#[test]
fn can_parse_crate_source_log_0_4_17() {
    let log = parse_canned_response("log").unwrap();
    let version = log.version("0.4.17".parse().unwrap()).unwrap();
    let _ = parse_canned_source(version).unwrap();
}

#[test]
fn subslice_offset_finds_subslices() {
    let data = b"hello world";
    assert_eq!(subslice_offset(data, &data[6..]), Some(6));
    assert_eq!(subslice_offset(data, &data[..5]), Some(0));
    assert_eq!(subslice_offset(&data[6..], &data[..5]), None);
    assert_eq!(subslice_offset(data, b"world"), None);
}

#[test]
fn routes_parse_and_display() {
    use crate::{version::VersionId, Route};
    use std::str::FromStr;

    let version = |v: &str| VersionId::from_str(v).unwrap();
    let cases = [
        ("/", Route::Home {}),
        ("/about", Route::About {}),
        (
            "/serde",
            Route::Crate {
                krate: "serde".into(),
            },
        ),
        (
            "/search/serde",
            Route::Search {
                query: "serde".into(),
            },
        ),
        (
            "/serde/1.0.0/1.0.1",
            Route::SingleSourceDiff {
                krate: "serde".into(),
                old_version: version("1.0.0"),
                new_version: version("1.0.1"),
            },
        ),
        (
            "/serde/1.0.0/1.0.1/src/de/mod.rs",
            Route::SingleSourceFile {
                krate: "serde".into(),
                old_version: version("1.0.0"),
                new_version: version("1.0.1"),
                path: "src/de/mod.rs".into(),
            },
        ),
        (
            "/log/0.4.17/tracing/0.1.0/Cargo.toml",
            Route::File {
                old_krate: "log".into(),
                old_version: version("0.4.17"),
                new_krate: "tracing".into(),
                new_version: version("0.1.0"),
                path: "Cargo.toml".into(),
            },
        ),
        (
            "/browse/serde/latest/src/lib.rs",
            Route::BrowseFile {
                krate: "serde".into(),
                version: version("latest"),
                path: "src/lib.rs".into(),
            },
        ),
        (
            "/repo/serde/1.0.0/files/src/lib.rs",
            Route::RepoFile {
                krate: "serde".into(),
                version: version("1.0.0"),
                path: "src/lib.rs".into(),
            },
        ),
    ];

    for (url, route) in cases {
        assert_eq!(Route::from_str(url).unwrap(), route, "parsing {url}");
        assert_eq!(route.to_string(), url, "displaying {route:?}");
    }

    assert!(matches!(
        Route::from_str("/a/b/c/d/e/f/g").unwrap(),
        Route::NotFound { .. }
    ));
}

fn source_with(path: &str, text: &str) -> std::sync::Arc<CrateSource> {
    let version = VersionInfo {
        checksum: vec![],
        dl_path: Default::default(),
        krate: "test".into(),
        yanked: false,
        version: "0.1.0".parse().unwrap(),
    };
    std::sync::Arc::new(CrateSource {
        version,
        files: [(path.into(), bytes::Bytes::from(text.to_owned()))].into(),
    })
}

fn file_changes(old: &str, new: &str) -> Changes {
    let diff = VersionDiff::new(
        source_with("src/lib.rs", old),
        source_with("src/lib.rs", new),
    );
    diff.files[camino::Utf8Path::new("src/lib.rs")].summary
}

#[test]
fn diff_ignores_line_ending_changes() {
    assert_eq!(
        file_changes("a\nb\nc\n", "a\r\nb\r\nc\r\n"),
        Changes::default()
    );
    assert_eq!(file_changes("a\r\nb\r\n", "a\nb\n"), Changes::default());
}

#[test]
fn diff_detects_changes_across_line_endings() {
    assert_eq!(
        file_changes("a\nb\nc\n", "a\r\nx\r\nc\r\n"),
        Changes {
            added: 1,
            removed: 1
        }
    );
}

#[test]
fn diff_detects_missing_trailing_newline() {
    assert_ne!(file_changes("a\nb\n", "a\nb"), Changes::default());
}
