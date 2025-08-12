use include_dir::include_dir;
use include_webdir::{CWebBundle, include_cwebdir};

const BUNDLE: CWebBundle = include_cwebdir!("$CARGO_MANIFEST_DIR/../examples/webapp/dist");
const EXPECTED: include_dir::Dir = include_dir!("$CARGO_MANIFEST_DIR/../examples/webapp/dist");

#[test]
fn test_index_html() {
    let file = BUNDLE.get("index.html").unwrap();
    let expected = EXPECTED.get_file("index.html").unwrap();
    assert_eq!(file.body, expected.contents());
    assert_eq!(file.etag, c"\"GFJ3N1HW33T4R\"");
    let expected_date = httpdate::fmt_http_date(expected.metadata().unwrap().modified());
    assert_eq!(file.last_modified.to_str().unwrap(), &expected_date);
    assert!(!file.immutable);
}

#[test]
fn test_script_js() {
    let file = BUNDLE.get("assets/script-abc123.js").unwrap();
    let expected = EXPECTED.get_file("assets/script-abc123.js").unwrap();
    assert_eq!(file.body, expected.contents());
    assert_eq!(file.etag, c"\"5Y1Y24E8PKPT6\"");
    let expected_date = httpdate::fmt_http_date(expected.metadata().unwrap().modified());
    assert_eq!(file.last_modified.to_str().unwrap(), &expected_date);
    assert!(file.immutable);
}
