use staged_builder::StagedBuilder;

#[derive(StagedBuilder, PartialEq, Debug)]
struct Foo {
    required: bool,
    #[builder(into)]
    required2: String,
    #[builder(default, into)]
    normal_default: String,
    #[builder(default(42))]
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

    let actual = foo::BuilderFinal::from(actual)
        .normal_default("b")
        .required(false)
        .build();
    let expected = Foo {
        required: false,
        required2: "a".to_string(),
        normal_default: "b".to_string(),
        custom_default: 42,
    };
    assert_eq!(actual, expected);
}
