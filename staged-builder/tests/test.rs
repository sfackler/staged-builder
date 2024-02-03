use staged_builder::{staged_builder, Validate};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;

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

#[staged_builder]
#[builder(validate)]
struct Validated {
    even: u32,
}

impl Validate for Validated {
    type Error = &'static str;

    fn validate(&self) -> Result<(), Self::Error> {
        if self.even % 2 == 0 {
            Ok(())
        } else {
            Err("is odd")
        }
    }
}

#[test]
fn validate() {
    Validated::builder().even(0).build().unwrap();
    Validated::builder().even(1).build().err().unwrap();
}

#[derive(PartialEq, Debug)]
#[staged_builder]
struct Collections {
    #[builder(list(item(type = u32)))]
    list: Vec<u32>,
    #[builder(set(item(type = &'static str)))]
    set: HashSet<&'static str>,
    #[builder(map(key(type = i32), value(type = bool)))]
    map: HashMap<i32, bool>,
}

#[test]
fn collections() {
    let actual = Collections::builder()
        .push_list(1)
        .push_list(2)
        .insert_set("hi")
        .insert_set("there")
        .insert_map(1, true)
        .insert_map(2, false)
        .build();
    let expected = Collections {
        list: vec![1, 2],
        set: HashSet::from(["hi", "there"]),
        map: HashMap::from([(1, true), (2, false)]),
    };
    assert_eq!(actual, expected);

    let actual = Collections::builder()
        .push_list(0)
        .list([1, 2])
        .set(["hi", "there"])
        .map([(1, true), (2, false)])
        .build();
    assert_eq!(actual, expected);

    let actual = Collections::builder()
        .push_list(1)
        .extend_list([2])
        .insert_set("hi")
        .extend_set(["there"])
        .insert_map(1, true)
        .extend_map([(2, false)])
        .build();
    assert_eq!(actual, expected);
}

#[derive(PartialEq, Debug)]
#[staged_builder]
struct CollectionsInto {
    #[builder(list(item(type = String, into)))]
    list: Vec<String>,
    #[builder(map(key(type = String, into), value(type = Option<i32>, into)))]
    map: HashMap<String, Option<i32>>,
}

#[test]
fn collections_into() {
    let actual = CollectionsInto::builder()
        .push_list("hi")
        .push_list("there")
        .insert_map("foo", 1)
        .insert_map("bar", None)
        .build();
    let expected = CollectionsInto {
        list: vec!["hi".to_string(), "there".to_string()],
        map: HashMap::from([("foo".to_string(), Some(1)), ("bar".to_string(), None)]),
    };
    assert_eq!(actual, expected);

    let actual = CollectionsInto::builder()
        .list(["hi", "there"])
        .map([("foo", Some(1)), ("bar", None)])
        .build();
    assert_eq!(actual, expected);

    let actual = CollectionsInto::builder()
        .push_list("hi")
        .extend_list(["there"])
        .insert_map("foo", 1)
        .extend_map([("bar", None)])
        .build();
    assert_eq!(actual, expected);
}

#[derive(PartialEq, Debug)]
#[staged_builder]
struct Custom {
    #[builder(custom(type = impl Display, convert = to_string))]
    string: String,
    #[builder(list(item(custom(type = impl Display, convert = to_string))))]
    list: Vec<String>,
}

fn to_string(value: impl Display) -> String {
    value.to_string()
}

#[test]
fn custom() {
    let actual = Custom::builder().string(42).push_list(true).build();
    let expected = Custom {
        string: "42".to_string(),
        list: vec!["true".to_string()],
    };
    assert_eq!(actual, expected);
}

#[staged_builder]
#[builder(mod = my_custom_mod)]
struct CustomMod {
    _foo: i32,
}

#[test]
fn custom_mod() {
    CustomMod::builder()._foo(1).build();
    my_custom_mod::Builder::default()._foo(1).build();
}

#[derive(PartialEq, Debug)]
#[staged_builder]
struct ClosureConvert {
    #[builder(custom(type = impl Display, convert = |s| s.to_string()))]
    single: String,
    #[builder(list(item(custom(type = impl Display, convert = |s| s.to_string()))))]
    list: Vec<String>,
}

#[test]
fn closure_convert() {
    let actual = ClosureConvert::builder().single(true).push_list(15).build();
    let expected = ClosureConvert {
        single: "true".to_string(),
        list: vec!["15".to_string()],
    };
    assert_eq!(actual, expected);
}

mod inline {
    use staged_builder::staged_builder;

    #[derive(PartialEq, Debug)]
    #[staged_builder]
    #[builder(inline)]
    struct Inline {
        a: i32,
    }

    #[test]
    fn inline() {
        let builder: Builder<AStage> = Inline::builder();
        let stage: Builder<Complete> = builder.a(1);
        let actual: Inline = stage.build();
        let expected = Inline { a: 1 };
        assert_eq!(actual, expected);
    }
}

#[derive(PartialEq, Debug)]
#[staged_builder]
#[builder(builder = MyBuilder, complete = MyComplete)]
struct CustomNames {
    #[builder(stage = MyAStage)]
    a: i32,
}

#[test]
fn custom_names() {
    let builder: custom_names::MyBuilder<custom_names::MyAStage> = CustomNames::builder();
    let stage: custom_names::MyBuilder<custom_names::MyComplete> = builder.a(1);
    let actual = stage.build();
    let expected = CustomNames { a: 1 };
    assert_eq!(actual, expected);
}

#[derive(PartialEq, Debug)]
#[staged_builder]
#[builder(update)]
struct Update {
    a: i32,
    #[builder(into)]
    b: String,
}

#[test]
fn update() {
    let v = Update::builder().a(1).b("hello").build();
    let actual = update::Builder::from(v).a(2).b("world").build();
    let expected = Update {
        a: 2,
        b: "world".to_string(),
    };
    assert_eq!(actual, expected);
}
