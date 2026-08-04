use super::camel_to_screaming_snake;

#[test]
fn camel_to_screaming_snake_basics() {
    assert_eq!(camel_to_screaming_snake("RealTime"), "REAL_TIME");
    assert_eq!(
        camel_to_screaming_snake("InteractiveHigh"),
        "INTERACTIVE_HIGH"
    );
    assert_eq!(camel_to_screaming_snake("Data"), "DATA");
    assert_eq!(camel_to_screaming_snake("Background"), "BACKGROUND");
}
