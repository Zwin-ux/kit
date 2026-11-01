#[test]
fn parses() {
    assert_eq!(kv::parse_kv(" a = 1 "), Some(("a", "1")));
    assert_eq!(kv::parse_kv("nope"), None);
}
