use crate::data::*;
use anyhow::Result;
use serde_json::from_reader;
use std::fs::File;

fn parse_canned_response(name: &str) -> Result<CrateResponse> {
    let response = File::open(format!("data/{name}.json"))?;
    let response: CrateResponse = from_reader(response)?;
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
