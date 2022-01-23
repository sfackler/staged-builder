use staged_builder::StagedBuilder;

#[derive(StagedBuilder)]
struct Foo {
    required: bool,
    required2: String,
    #[builder(default)]
    normal_default: String,
    #[builder(default(42))]
    custom_default: i32,
}
