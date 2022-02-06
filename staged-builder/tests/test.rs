use staged_builder::staged_builder;

#[derive(PartialEq, Debug)]
#[staged_builder]
struct Foo {
    required: bool,
    #[builder(into)]
    required2: String,
    #[builder(default, into)]
    normal_default: String,
    #[builder(default = 42)]
    custom_default: i32,
}

#[test]
fn basic() {
    let actual = Foo::builder().required(true).required2("a").build();
    let expected = Foo {
        required: true,
        required2: "a".to_string(),
        normal_default: "".to_string(),
        custom_default: 42,
    };
    assert_eq!(actual, expected);
}