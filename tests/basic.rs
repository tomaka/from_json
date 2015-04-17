#[macro_use]
extern crate from_json;

struct Test {
    a: isize,
    b: bool,
    c: Test2,
    d: Option<String>,
}

derive_from_json!(Test, a, b as "real_b", c, d);

struct Test2 {
    e: String,
    f: Option<bool>,
}

derive_from_json!(Test2, e, f);

#[test]
fn test() {
    use from_json::FromJson;

    let json = from_json::Json::from_str(r#"{ "a": 5, "real_b": true, "c": { "e": "hello", "f": false } }"#).unwrap();

    let content: Test = FromJson::from_json(&json).unwrap();

    assert_eq!(content.a, 5);
    assert_eq!(content.b, true);
    assert_eq!(content.c.e, "hello".to_string());
    assert_eq!(content.c.f, Some(false));
    assert_eq!(content.d, None);
}
