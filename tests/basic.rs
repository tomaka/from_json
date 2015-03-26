#![feature(custom_attribute)]
#![feature(plugin)]

#![plugin(from_json_macros)]

extern crate from_json;
extern crate rustc_serialize as serialize;

#[from_json_struct]
struct Test {
    a: isize,
    #[from_json_name = "real_b"]
    b: bool,
    c: Test2,
    d: Option<String>,
}

#[from_json_struct]
struct Test2 {
    e: String,
    f: Option<bool>,
}

#[test]
fn test() {
    use from_json::FromJson;

    let json = serialize::json::Json::from_str(r#"{ "a": 5, "real_b": true, "c": { "e": "hello", "f": false } }"#).unwrap();

    let content: Test = FromJson::from_json(&json).unwrap();

    assert_eq!(content.a, 5);
    assert_eq!(content.b, true);
    assert_eq!(content.c.e, "hello".to_string());
    assert_eq!(content.c.f, Some(false));
    assert_eq!(content.d, None);
}
