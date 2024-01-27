use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use structmeta::{NameArgs, NameValue, StructMeta};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Error, Expr, Field, Fields, FieldsNamed,
    Ident, Type, Visibility,
};

/// Creates a staged builder interface for structs.
///
/// The macro will create a submodule with the `snake_case` version of the type's name containing the builder types, and
/// add a `builder` constructor function to the type. Each required field of the struct will correspond to a builder
/// type named after it, with an additional "final" stage to set optional fields and construct the final value.
///
/// By default, all fields are considered required and their setters will simply take their declared type by-value. This
/// behavior can be customized with field options.
///
/// # Struct options
///
/// Options can be applied at the struct level via the `#[builder(...)]` attribute as a comma-separated sequence:
///
/// * `validate` - The final `build` method will return a `Result`, calling the type's `Validate` implementation before
///     returning the constructed value.
///
/// # Field options
///
/// Options can be applied to individual fields via the `#[builder(...)]` attribute as a comma-separated sequence:
///
/// * `default` - Causes the field to be considered optional. The [`Default`] trait is normally used to generate the
///     default field value. A custom default can be specified with `default = <expr>`, where `<expr>` is an expression.
/// * `into` - Causes the setter method for the field to take `impl Into<FieldType>` rather than `FieldType` directly.
/// * `custom` - Causes the setter method to perform an arbitrary conversion for the field. The option expects a `type`
///     which will be used as the argument type in the setter, and a `convert` callable expression which will be invoked
///     by the setter. For example, the annotation `#[builder(into)]` on a field of type `T` is equivalent to the
///     annotation `#[builder(custom(type = impl Into<T>, convert = Into::into))]`.
/// * `list` - Causes the field to be treated as a "list style" type. It will default to an empty collection, and three
///     setter methods will be generated: `push_foo` to add a single value, `foo` to set the contents, and `extend_foo`
///     to exend the collection with new values. The underlying type must have a `push` method, a [`FromIterator`]
///     implementation, and an [`Extend`] implementation. The item type must be configured in the attribute:
///     `#[builder(list(item(type = YourItemType)))]`.
/// * `set` - Causes the field to be treated as a "set style" type. It will default to an empty collection, and three
///     setter methods will be generated: `insert_foo` to add a single value, `foo` to set the contents, and
///     `extend_foo` to exend the collection with new values. The underlying type must have an `insert` method, a
///     [`FromIterator`] implementation, and an [`Extend`] implementation. The item type must be configured in the
///     attribute: `#[builder(set(item(type = YourItemType)))]`.
/// * `map` - Causes the field to be treated as a "map style" type. It will default to an empty collection, and three
///     setter methods will be generated: `insert_foo` to add a single entry, `foo` to set the contents, and
///     `extend_foo` to exend the collection with new entries. The underlying type must have an `insert` method, a
///     [`FromIterator`] implementation, and an [`Extend`] implementation. The key and value types must be configured in
///     the attribute: `#[builder(map(key(type = YourKeyType), value(type = YourValueType)))]`.
///
/// # Collection type options
///
/// Options can be applied to the item types of collections as a comma-separated sequence:
///
/// * `type` - Indicates the type of the item in the collection. Required unless using `custom`.
/// * `into` - Causes setter methods to take `impl<Into<ItemType>>` rather than `ItemType` directly.
/// * `custom` - Causes the setter methods to perform an arbitrary conversion for the field.
///
/// # Example expansion
///
/// ```ignore
/// use staged_builder::staged_builder;
///
/// #[staged_builder]
/// pub struct MyStruct {
///     required_field: u32,
///     #[builder(into)]
///     into_required_field: String,
///     #[builder(default)]
///     standard_optional_field: bool,
///     #[builder(default = "foobar".to_string())]
///     custom_default_field: String,
///     #[builder(list(item(type = i32)))]
///     list_field: Vec<i32>,
/// }
/// ```
///
/// Will expand into:
///
/// ```ignore
/// pub struct MyStruct {
///     required_field: u32,
///     into_required_field: String,
///     standard_optional_field: bool,
///     custom_default_field: String,
///     list_field: Vec<i32>,
/// }
///
/// impl MyStruct {
///     pub fn builder() -> my_struct::Builder<my_struct::RequiredFieldStage> {
///         // ...
///     }
/// }
///
/// pub mod my_struct {
///     pub struct Builder<T> {
///         // ...
///     }
///
///     impl Default for Builder<RequiredFieldStage> {
///         fn default() -> Self {
///             // ...
///         }
///     }
///
///     impl Builder<RequiredFieldStage> {
///         pub fn required_field(self, required_field: u32) -> Builder<IntoRequiredFieldStage> {
///             // ...
///         }
///     }
///
///     impl Builder<IntoRequiredFieldStage> {
///         pub fn into_required_field(self, into_required_field: impl Into<String>) -> Builder<FinalStage> {
///             // ...
///         }
///     }
///
///     impl Builder<FinalStage> {
///         pub fn standard_optional_field(self, standard_optional_field: bool) -> Self {
///             // ...
///         }
///
///         pub fn custom_default_field(self, custom_default_field: String) -> Self {
///             // ...
///         }
///
///         pub fn push_list_field(self, list_field: i32) -> Self {
///             // ...
///         }
///
///         pub fn list_field(self, list_field: impl IntoIterator<Item = i32>) -> Self {
///             // ...
///         }
///
///         pub fn extend_list_field(self, list_field: impl IntoIterator<Item = i32>) -> Self {
///             // ...
///         }
///
///         pub fn build(self) -> super::MyStruct {
///             // ...
///         }
///     }
///
///     pub struct RequiredFieldStage {
///         // ...
///     }
///
///     pub struct IntoRequiredFieldStage {
///         // ...
///     }
///
///     pub struct FinalStage {
///         // ...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn staged_builder(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as AttrInput);

    let attrs = input.attrs;
    let body = input.body;
    quote! {
        #[derive(::staged_builder::__StagedBuilderInternalDerive)]
        #(#attrs)*
        #body
    }
    .into()
}

struct AttrInput {
    attrs: Vec<Attribute>,
    body: TokenStream,
}

impl Parse for AttrInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let body = input.parse()?;

        Ok(AttrInput { attrs, body })
    }
}

// Not public API.
#[doc(hidden)]
#[proc_macro_derive(__StagedBuilderInternalDerive, attributes(builder))]
pub fn __internal_derive_staged_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

fn expand(input: DeriveInput) -> Result<TokenStream, Error> {
    let struct_ = match &input.data {
        Data::Struct(struct_) => struct_,
        _ => {
            return Err(Error::new_spanned(
                &input,
                "staged builders can only be derived on structs",
            ))
        }
    };

    let fields = match &struct_.fields {
        Fields::Named(fields) => fields,
        _ => {
            return Err(Error::new_spanned(
                &input,
                "staged builders cannot be derived on tuple or unit structs",
            ))
        }
    };

    let overrides = StructOverrides::new(&input.attrs)?;
    let fields = resolve_fields(fields)?;

    let vis = &input.vis;
    let module_name = module_name(&input);

    let builder_impl = builder_impl(&input, &fields);

    let module_docs = format!("Builder types for [`{}`].", &input.ident);

    let builder = builder(&input);
    let default = default_impl(&fields);
    let stages = fields
        .iter()
        .enumerate()
        .filter(|(_, f)| f.default.is_none())
        .map(|(i, _)| stage(&input, i, &fields));
    let final_stage = final_stage(&input, &overrides, &fields);

    let tokens = quote! {
        #builder_impl

        #[doc = #module_docs]
        #vis mod #module_name {
            use super::*;

            #builder
            #default
            #(#stages)*
            #final_stage
        }
    };

    Ok(tokens)
}

fn module_name(input: &DeriveInput) -> Ident {
    Ident::new(&input.ident.to_string().to_snake_case(), input.ident.span())
}

fn builder_impl(input: &DeriveInput, fields: &[ResolvedField<'_>]) -> TokenStream {
    let name = &input.ident;
    let vis = &input.vis;

    let module_name = module_name(input);

    let builder_name = initial_stage(fields).unwrap_or_else(final_name);

    quote! {
        impl #name {
            /// Returns a new builder.
            #[inline]
            #vis fn builder() -> #module_name::Builder<#module_name::#builder_name> {
                ::staged_builder::__private::Default::default()
            }
        }
    }
}

fn initial_stage(fields: &[ResolvedField<'_>]) -> Option<Ident> {
    fields
        .iter()
        .find(|f| f.default.is_none())
        .map(|f| stage_name(f))
}

fn builder(input: &DeriveInput) -> TokenStream {
    let docs = format!("A builder for [{0}](super::{0}).", input.ident);

    quote! {
        #[doc = #docs]
        pub struct Builder<T>(T);
    }
}

fn default_impl(fields: &[ResolvedField<'_>]) -> TokenStream {
    let (stage, initializers) = match initial_stage(fields) {
        Some(stage) => (stage, quote!()),
        None => (final_name(), default_field_initializers(fields)),
    };

    quote! {
        impl ::staged_builder::__private::Default for Builder<#stage> {
            #[inline]
            fn default() -> Self {
                Builder(#stage {
                    #initializers
                })
            }
        }
    }
}

fn default_field_initializers(fields: &[ResolvedField<'_>]) -> TokenStream {
    let fields = fields.iter().filter_map(|f| {
        f.default.as_ref().map(|default| {
            let name = f.field.ident.as_ref().unwrap();
            quote!(#name: #default)
        })
    });

    quote!(#(#fields,)*)
}

fn stage(input: &DeriveInput, idx: usize, fields: &[ResolvedField<'_>]) -> TokenStream {
    let vis = stage_vis(&input.vis);
    let field = &fields[idx];
    let name = field.field.ident.as_ref().unwrap();

    let (type_, assign) = match &field.mode {
        FieldMode::Normal { type_, assign } => (type_, assign),
        _ => unreachable!(),
    };

    let builder_name = stage_name(field);

    let existing_fields = fields[..idx]
        .iter()
        .filter(|f| f.default.is_none())
        .collect::<Vec<_>>();

    let existing_names = existing_fields
        .iter()
        .map(|f| f.field.ident.as_ref().unwrap())
        .collect::<Vec<_>>();
    let existing_types = existing_fields.iter().map(|f| &f.field.ty);

    let (next_builder, optional_fields) =
        match fields[idx + 1..].iter().find(|f| f.default.is_none()) {
            Some(field) => (stage_name(field), quote!()),
            None => (final_name(), default_field_initializers(fields)),
        };

    let struct_docs = format!("The `{name}` stage for [`Builder`].");
    let setter_docs = format!("Sets the `{name}` field.");

    quote! {
        #[doc = #struct_docs]
        #vis struct #builder_name {
            #(#existing_names: #existing_types,)*
        }

        impl Builder<#builder_name> {
            #[doc = #setter_docs]
            #[inline]
            pub fn #name(self, #name: #type_) -> Builder<#next_builder> {
                Builder(#next_builder {
                    #(#existing_names: self.0.#existing_names,)*
                    #name: #assign,
                    #optional_fields
                })
            }
        }
    }
}

fn stage_vis(vis: &Visibility) -> TokenStream {
    match vis {
        Visibility::Public(_) => quote!(#vis),
        Visibility::Restricted(restricted) => {
            let path = &restricted.path;
            if path.leading_colon.is_some()
                || path.segments.first().map_or(false, |i| i.ident == "crate")
            {
                quote!(#vis)
            } else if restricted.path.is_ident("self") {
                quote!(pub (super))
            } else {
                let path = &restricted.path;
                quote!(pub (in super::#path))
            }
        }
        Visibility::Inherited => quote!(pub (super)),
    }
}

fn stage_name(field: &ResolvedField<'_>) -> Ident {
    let name = format!(
        "{}Stage",
        field
            .field
            .ident
            .as_ref()
            .unwrap()
            .to_string()
            .to_upper_camel_case()
    );
    Ident::new(&name, field.field.span())
}

fn final_name() -> Ident {
    Ident::new("Complete", Span::call_site())
}

fn final_stage(
    input: &DeriveInput,
    overrides: &StructOverrides,
    fields: &[ResolvedField<'_>],
) -> TokenStream {
    let vis = stage_vis(&input.vis);
    let builder_name = final_name();
    let struct_name = &input.ident;
    let names = fields.iter().map(|f| f.field.ident.as_ref().unwrap());
    let types = fields.iter().map(|f| &f.field.ty).collect::<Vec<_>>();

    let struct_docs = format!("The final stage for [`{struct_name}`](super::{struct_name}).");

    let setters = fields
        .iter()
        .filter(|f| f.default.is_some())
        .map(final_stage_setter);

    let build_docs =
        format!("Consumes the builder, returning a [`{struct_name}`](super::{struct_name}).");

    let build = if overrides.validate {
        validated_build(input, fields)
    } else {
        unvalidated_build(input, fields)
    };

    quote! {
        #[doc = #struct_docs]
        #vis struct #builder_name {
            #(#names: #types,)*
        }

        impl Builder<#builder_name> {
            #(#setters)*

            #[doc = #build_docs]
            #build
        }
    }
}

fn final_stage_setter(field: &ResolvedField<'_>) -> TokenStream {
    let name = field.field.ident.as_ref().unwrap();

    match &field.mode {
        FieldMode::Normal { type_, assign } => {
            let docs = format!("Sets the `{name}` field.");
            quote! {
                #[doc = #docs]
                #[inline]
                pub fn #name(mut self, #name: #type_) -> Self {
                    self.0.#name = #assign;
                    self
                }
            }
        }
        FieldMode::Seq { push, item } => {
            let type_ = &item.type_;
            let convert = item.convert(name);
            let convert_iter = item.convert_iter(name);

            let push_docs = format!("Adds a value to the `{name}` field.");
            let push_method = Ident::new(&format!("{push}_{name}"), name.span());

            let docs = format!("Sets the `{name}` field.");

            let extend_docs = format!("Adds values to the `{name}` field.");
            let extend_method = Ident::new(&format!("extend_{name}"), name.span());

            quote! {
                #[doc = #push_docs]
                #[inline]
                pub fn #push_method(mut self, #name: #type_) -> Self {
                    self.0.#name.#push(#convert);
                    self
                }

                #[doc = #docs]
                #[inline]
                pub fn #name(
                    mut self,
                    #name: impl ::staged_builder::__private::IntoIterator<Item = #type_>,
                ) -> Self
                {
                    self.0.#name = ::staged_builder::__private::FromIterator::from_iter(#convert_iter);
                    self
                }

                #[doc = #extend_docs]
                #[inline]
                pub fn #extend_method(
                    mut self,
                    #name: impl ::staged_builder::__private::IntoIterator<Item = #type_>,
                ) -> Self
                {
                    ::staged_builder::__private::Extend::extend(&mut self.0.#name, #convert_iter);
                    self
                }
            }
        }
        FieldMode::Map { key, value } => {
            let key_name = Ident::new("key", Span::call_site());
            let key_type = &key.type_;
            let key_convert = key.convert(&key_name);

            let value_name = Ident::new("value", Span::call_site());
            let value_type = &value.type_;
            let value_convert = value.convert(&value_name);

            let iter_convert = if key.convert.is_some() || value.convert.is_some() {
                quote! {
                    ::staged_builder::__private::Iterator::map(
                        ::staged_builder::__private::IntoIterator::into_iter(#name),
                        |(#key_name, #value_name)| (#key_convert, #value_convert)
                    )
                }
            } else {
                quote!(#name)
            };

            let insert_docs = format!("Adds an entry to the `{name}` field.");
            let insert_method = Ident::new(&format!("insert_{name}"), name.span());

            let docs = format!("Sets the `{name}` field.");

            let extend_docs = format!("Adds entries to the `{name}` field.");
            let extend_method = Ident::new(&format!("extend_{name}"), name.span());

            quote! {
                #[doc = #insert_docs]
                #[inline]
                pub fn #insert_method(mut self, #key_name: #key_type, #value_name: #value_type) -> Self {
                    self.0.#name.insert(#key_convert, #value_convert);
                    self
                }

                #[doc = #docs]
                #[inline]
                pub fn #name(
                    mut self,
                    #name: impl ::staged_builder::__private::IntoIterator<Item = (#key_type, #value_type)>,
                ) -> Self {
                    self.0.#name = ::staged_builder::__private::FromIterator::from_iter(#iter_convert);
                    self
                }

                #[doc = #extend_docs]
                #[inline]
                pub fn #extend_method(
                    mut self,
                    #name: impl ::staged_builder::__private::IntoIterator<Item = (#key_type, #value_type)>,
                ) -> Self
                {
                    ::staged_builder::__private::Extend::extend(&mut self.0.#name, #iter_convert);
                    self
                }
            }
        }
    }
}

fn validated_build(input: &DeriveInput, fields: &[ResolvedField<'_>]) -> TokenStream {
    let struct_name = &input.ident;
    let names = fields
        .iter()
        .map(|f| f.field.ident.as_ref().unwrap())
        .collect::<Vec<_>>();

    quote! {
        #[inline]
        pub fn build(
            self,
        ) -> ::staged_builder::__private::Result<
            super::#struct_name,
            <super::#struct_name as ::staged_builder::Validate>::Error,
        > {
            let value = super::#struct_name {
                #(#names: self.0.#names,)*
            };
            ::staged_builder::Validate::validate(&value)?;
            ::staged_builder::__private::Result::Ok(value)
        }
    }
}

fn unvalidated_build(input: &DeriveInput, fields: &[ResolvedField<'_>]) -> TokenStream {
    let struct_name = &input.ident;
    let names = fields
        .iter()
        .map(|f| f.field.ident.as_ref().unwrap())
        .collect::<Vec<_>>();

    quote! {
        #[inline]
        pub fn build(self) -> super::#struct_name {
            super::#struct_name {
                #(#names: self.0.#names,)*
            }
        }
    }
}

fn resolve_fields(fields: &FieldsNamed) -> Result<Vec<ResolvedField<'_>>, Error> {
    let mut resolved_fields = vec![];
    let mut error = None::<Error>;

    for field in &fields.named {
        match ResolvedField::new(field) {
            Ok(field) => resolved_fields.push(field),
            Err(e) => match &mut error {
                Some(error) => error.combine(e),
                None => error = Some(e),
            },
        }
    }

    match error {
        Some(error) => Err(error),
        None => Ok(resolved_fields),
    }
}

#[derive(StructMeta, Default)]
struct StructOverrides {
    validate: bool,
}

impl StructOverrides {
    fn new(attrs: &[Attribute]) -> Result<Self, Error> {
        attrs
            .iter()
            .filter(|a| a.meta.path().is_ident("builder"))
            .map(|a| a.parse_args())
            .next()
            .transpose()
            .map(|o| o.unwrap_or_default())
    }
}

struct ResolvedField<'a> {
    field: &'a Field,
    default: Option<TokenStream>,
    mode: FieldMode,
}

enum FieldMode {
    Normal {
        type_: TokenStream,
        assign: TokenStream,
    },
    Seq {
        push: TokenStream,
        item: ParamConfig,
    },
    Map {
        key: ParamConfig,
        value: ParamConfig,
    },
}

struct ParamConfig {
    type_: TokenStream,
    convert: Option<TokenStream>,
}

impl ParamConfig {
    fn new(overrides: &NameArgs<ParamOverrides>) -> Result<Self, Error> {
        match &overrides.args.custom {
            Some(custom) => {
                let type_ = &custom.args.type_;
                let convert = &custom.args.convert;
                Ok(ParamConfig {
                    type_: quote!(#type_),
                    convert: Some(quote!(#convert)),
                })
            }
            None => {
                let type_ = overrides.args.type_.as_ref().ok_or_else(|| {
                    Error::new(overrides.name_span, "missing `type` configuration")
                })?;
                let mut type_ = quote!(#type_);
                let mut convert = None;
                if overrides.args.into {
                    type_ = quote!(impl ::staged_builder::__private::Into<#type_>);
                    convert = Some(quote!(::staged_builder::__private::Into::into));
                }

                Ok(ParamConfig { type_, convert })
            }
        }
    }

    fn convert(&self, name: &Ident) -> TokenStream {
        match &self.convert {
            Some(convert_fn) => quote!(#convert_fn(#name)),
            None => quote!(#name),
        }
    }

    fn convert_iter(&self, name: &Ident) -> TokenStream {
        match &self.convert {
            Some(convert_fn) => quote! {
                ::staged_builder::__private::Iterator::map(
                    ::staged_builder::__private::IntoIterator::into_iter(#name),
                    #convert_fn,
                )
            },
            None => quote!(#name),
        }
    }
}

impl<'a> ResolvedField<'a> {
    fn new(field: &'a Field) -> Result<ResolvedField<'a>, Error> {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        let mut resolved = ResolvedField {
            field,
            default: None,
            mode: FieldMode::Normal {
                type_: quote!(#ty),
                assign: quote!(#name),
            },
        };

        let overrides = FieldOverrides::new(&field.attrs)?;

        if let Some(default) = overrides.default {
            let default = match default.value {
                Some(v) => quote!(#v),
                None => quote!(::staged_builder::__private::Default::default()),
            };
            resolved.default = Some(default)
        }

        if overrides.into {
            resolved.mode = FieldMode::Normal {
                type_: quote!(impl ::staged_builder::__private::Into<#ty>),
                assign: quote!(#name.into()),
            }
        } else if let Some(custom) = overrides.custom {
            let type_ = custom.args.type_;
            let convert = custom.args.convert;
            resolved.mode = FieldMode::Normal {
                type_: quote!(#type_),
                assign: quote!(#convert(#name)),
            }
        } else if let Some(list) = overrides.list {
            if resolved.default.is_none() {
                resolved.default = Some(quote!(::staged_builder::__private::Default::default()));
            }
            resolved.mode = FieldMode::Seq {
                push: quote!(push),
                item: ParamConfig::new(&list.args.item)?,
            }
        } else if let Some(set) = overrides.set {
            if resolved.default.is_none() {
                resolved.default = Some(quote!(::staged_builder::__private::Default::default()));
            }
            resolved.mode = FieldMode::Seq {
                push: quote!(insert),
                item: ParamConfig::new(&set.args.item)?,
            }
        } else if let Some(map) = overrides.map {
            if resolved.default.is_none() {
                resolved.default = Some(quote!(::staged_builder::__private::Default::default()));
            }
            resolved.mode = FieldMode::Map {
                key: ParamConfig::new(&map.args.key)?,
                value: ParamConfig::new(&map.args.value)?,
            }
        }

        Ok(resolved)
    }
}

#[derive(StructMeta, Default)]
struct FieldOverrides {
    default: Option<NameValue<Option<Expr>>>,
    into: bool,
    custom: Option<NameArgs<CustomOverrides>>,
    list: Option<NameArgs<SeqOverrides>>,
    set: Option<NameArgs<SeqOverrides>>,
    map: Option<NameArgs<MapOverrides>>,
}

impl FieldOverrides {
    fn new(attrs: &[Attribute]) -> Result<Self, Error> {
        attrs
            .iter()
            .filter(|a| a.meta.path().is_ident("builder"))
            .map(|a| a.parse_args())
            .next()
            .transpose()
            .map(|o| o.unwrap_or_default())
    }
}

#[derive(StructMeta)]
struct CustomOverrides {
    #[struct_meta(name = "type")]
    type_: Type,
    convert: Expr,
}

#[derive(StructMeta)]
struct SeqOverrides {
    item: NameArgs<ParamOverrides>,
}

#[derive(StructMeta)]
struct ParamOverrides {
    #[struct_meta(name = "type")]
    type_: Option<Type>,
    into: bool,
    custom: Option<NameArgs<CustomOverrides>>,
}

#[derive(StructMeta)]
struct MapOverrides {
    key: NameArgs<ParamOverrides>,
    value: NameArgs<ParamOverrides>,
}
