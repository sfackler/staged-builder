use staged_builder::{staged_builder, Validate};
use std::collections::HashSet;

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
}

#[test]
fn collections() {
    let actual = Collections::builder()
        .push_list(1)
        .push_list(2)
        .insert_set("hi")
        .insert_set("there")
        .build();
    let expected = Collections {
        list: vec![1, 2],
        set: HashSet::from(["hi", "there"]),
    };
    assert_eq!(actual, expected);

    let actual = Collections::builder()
        .push_list(0)
        .list([1, 2])
        .set(["hi", "there"])
        .build();
    assert_eq!(actual, expected);

    let actual = Collections::builder()
        .push_list(1)
        .extend_list([2])
        .insert_set("hi")
        .extend_set(["there"])
        .build();
    assert_eq!(actual, expected);
}
