use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{
    parenthesized, parse_macro_input, Attribute, Data, DeriveInput, Error, Expr, Field, Fields,
    FieldsNamed, Ident, Token, Type, Visibility,
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
/// * `into` - Causes the setter method for the field to take `impl Into<FieldType>` rather than `FieldType` directly.
/// * `default` - Causes the field to be considered optional. The [`Default`] trait is normally used to generate the
///     default field value. A custom default can be specified with `default = <expr>`, where `<expr>` is an expression.
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
///     custom_default_field: i32,
/// }
///
/// impl MyStruct {
///     pub fn builder() -> my_struct::BuilderRequiredFieldStage {
///         // ...
///     }
/// }
///
/// pub struct my_struct {
///     pub struct BuilderRequiredFieldStage {}
///
///     impl BuilderRequiredFieldStage {
///         pub fn required_field(self, required_field: u32) -> BuilderIntoRequiredFieldStage {
///             // ...
///         }
///     }
///
///     pub struct BuilderIntoRequiredFieldStage {
///         // ...
///     }
///
///     impl BuilderIntoRequiredFieldStage {
///         pub fn into_required_field(self, into_required_field: impl Into<String>) -> BuilderFinal {
///             // ...
///         }
///     }
///
///     pub struct BuilderFinal {
///         // ...
///     }
///
///     impl BuilderFinal {
///         pub fn standard_optional_field(self, standard_optional_field: bool) -> Self {
///             // ...
///         }
///
///         pub fn custom_default_field(self, custom_default_field: String) -> Self {
///             // ...
///         }
///
///         pub fn build(self) -> super::MyStruct {
///             // ...
///         }
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

    let (builder_name, fields) = match fields.iter().find(|f| f.default.is_none()) {
        Some(f) => (stage_name(f), quote!()),
        None => (final_name(), default_field_initializers(fields)),
    };

    quote! {
        impl #name {
            /// Returns a new builder.
            #[inline]
            #vis fn builder() -> #module_name::#builder_name {
                #module_name::#builder_name {
                    #fields
                }
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

    let struct_docs = format!(
        "The `{name}` stage builder for [`{0}`](super::{0}).",
        input.ident
    );
    let setter_docs = format!("Sets the `{name}` field.");

    quote! {
        #[doc = #struct_docs]
        #vis struct #builder_name {
            #(#existing_names: #existing_types,)*
        }

        impl #builder_name {
            #[doc = #setter_docs]
            #[inline]
            pub fn #name(self, #name: #type_) -> #next_builder {
                #next_builder {
                    #(#existing_names: self.#existing_names,)*
                    #name: #assign,
                    #optional_fields
                }
            }
        }
    }
}

fn stage_vis(vis: &Visibility) -> TokenStream {
    match vis {
        Visibility::Public(_) | Visibility::Crate(_) => quote!(#vis),
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
        "Builder{}Stage",
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
    Ident::new("BuilderFinal", Span::call_site())
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

    let struct_docs =
        format!("The final stage builder for [`{struct_name}`](super::{struct_name}).");

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
            #(pub(super) #names: #types,)*
        }

        impl #builder_name {
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
                    self.#name = #assign;
                    self
                }
            }
        }
        FieldMode::UnaryCollection { kind, item } => {
            let push_setter = match kind {
                UnaryKind::List => Ident::new("push", name.span()),
                UnaryKind::Set => Ident::new("insert", name.span()),
            };

            let push_docs = format!("Adds a value to the `{name}` field.");
            let push_method = Ident::new(&format!("{push_setter}_{name}"), name.span());

            let docs = format!("Sets the `{name}` field.");

            let extend_docs = format!("Adds values to the `{name}` field.");
            let extend_method = Ident::new(&format!("extend_{name}"), name.span());

            quote! {
                #[doc = #push_docs]
                #[inline]
                pub fn #push_method(mut self, #name: #item) -> Self {
                    self.#name.#push_setter(#name);
                    self
                }

                #[doc = #docs]
                #[inline]
                pub fn #name(
                    mut self,
                    #name: impl staged_builder::__private::IntoIterator<Item = #item>,
                ) -> Self
                {
                    self.#name = staged_builder::__private::FromIterator::from_iter(#name);
                    self
                }

                #[doc = #extend_docs]
                #[inline]
                pub fn #extend_method(
                    mut self,
                    #name: impl staged_builder::__private::IntoIterator<Item = #item>,
                ) -> Self
                {
                    staged_builder::__private::Extend::extend(&mut self.#name, #name);
                    self
                }
            }
        }
        FieldMode::Map { key, value } => {
            let insert_docs = format!("Adds an entry to the `{name}` field.");
            let insert_method = Ident::new(&format!("insert_{name}"), name.span());

            let docs = format!("Sets the `{name}` field.");

            let extend_docs = format!("Adds entries to the `{name}` field.");
            let extend_method = Ident::new(&format!("extend_{name}"), name.span());

            quote! {
                #[doc = #insert_docs]
                #[inline]
                pub fn #insert_method(mut self, key: #key, value: #value) -> Self {
                    self.#name.insert(key, value);
                    self
                }

                #[doc = #docs]
                #[inline]
                pub fn #name(
                    mut self,
                    #name: impl staged_builder::__private::IntoIterator<Item = (#key, #value)>,
                ) -> Self {
                    self.#name = staged_builder::__private::FromIterator::from_iter(#name);
                    self
                }

                #[doc = #extend_docs]
                #[inline]
                pub fn #extend_method(
                    mut self,
                    #name: impl staged_builder::__private::IntoIterator<Item = (#key, #value)>,
                ) -> Self
                {
                    staged_builder::__private::Extend::extend(&mut self.#name, #name);
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
        ) -> staged_builder::__private::Result<
            super::#struct_name,
            <super::#struct_name as staged_builder::Validate>::Error,
        > {
            let value = super::#struct_name {
                #(#names: self.#names,)*
            };
            staged_builder::Validate::validate(&value)?;
            staged_builder::__private::Result::Ok(value)
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
                #(#names: self.#names,)*
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

struct StructOverrides {
    validate: bool,
}

impl StructOverrides {
    fn new(attrs: &[Attribute]) -> Result<Self, Error> {
        let mut overrides = StructOverrides { validate: false };

        for attr in attrs {
            if !attr.path.is_ident("builder") {
                continue;
            }

            let parsed = attr.parse_args_with(|p: ParseStream<'_>| {
                p.parse_terminated::<_, Token![,]>(StructOverride::parse)
            })?;

            for override_ in parsed {
                match override_ {
                    StructOverride::Validate => overrides.validate = true,
                }
            }
        }

        Ok(overrides)
    }
}

enum StructOverride {
    Validate,
}

impl Parse for StructOverride {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let name = input.parse::<Ident>()?;
        if name == "validate" {
            Ok(StructOverride::Validate)
        } else {
            Err(Error::new(name.span(), "expected `validate`"))
        }
    }
}

enum FieldMode {
    Normal {
        type_: TokenStream,
        assign: TokenStream,
    },
    UnaryCollection {
        kind: UnaryKind,
        item: TokenStream,
    },
    Map {
        key: TokenStream,
        value: TokenStream,
    },
}

enum UnaryKind {
    List,
    Set,
}

struct ResolvedField<'a> {
    field: &'a Field,
    default: Option<TokenStream>,
    mode: FieldMode,
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

        for attr in &field.attrs {
            if !attr.path.is_ident("builder") {
                continue;
            }

            let overrides = attr.parse_args_with(|p: ParseStream<'_>| {
                p.parse_terminated::<_, Token![,]>(FieldOverride::parse)
            })?;

            for override_ in overrides {
                match override_ {
                    FieldOverride::Default { expr } => {
                        resolved.default = Some(
                            expr.unwrap_or(quote!(staged_builder::__private::Default::default())),
                        );
                    }
                    FieldOverride::Into => {
                        resolved.mode = FieldMode::Normal {
                            type_: quote!(impl staged_builder::__private::Into<#ty>),
                            assign: quote!(#name.into()),
                        };
                    }
                    FieldOverride::UnaryCollection { kind, item } => {
                        resolved.default =
                            Some(quote!(staged_builder::__private::Default::default()));
                        resolved.mode = FieldMode::UnaryCollection { kind, item };
                    }
                    FieldOverride::Map { key, value } => {
                        resolved.default =
                            Some(quote!(staged_builder::__private::Default::default()));
                        resolved.mode = FieldMode::Map { key, value };
                    }
                }
            }
        }

        Ok(resolved)
    }
}

enum FieldOverride {
    Default {
        expr: Option<TokenStream>,
    },
    Into,
    UnaryCollection {
        kind: UnaryKind,
        item: TokenStream,
    },
    Map {
        key: TokenStream,
        value: TokenStream,
    },
}

impl Parse for FieldOverride {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let name = input.parse::<Ident>()?;
        if name == "default" {
            let expr = if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;
                let expr = input.parse::<Expr>()?;

                Some(expr.to_token_stream())
            } else {
                None
            };

            Ok(FieldOverride::Default { expr })
        } else if name == "into" {
            Ok(FieldOverride::Into)
        } else if name == "list" || name == "set" {
            let content;
            parenthesized!(content in input);

            let mut item = None;
            for override_ in content.parse_terminated::<_, Token![,]>(UnaryOverride::parse)? {
                match override_ {
                    UnaryOverride::Item { type_ } => {
                        item = Some(type_);
                    }
                }
            }

            let kind = if name == "list" {
                UnaryKind::List
            } else {
                UnaryKind::Set
            };
            let item =
                item.ok_or_else(|| Error::new(name.span(), "missing `item` configuration"))?;
            Ok(FieldOverride::UnaryCollection { kind, item })
        } else if name == "map" {
            let content;
            parenthesized!(content in input);

            let mut key = None;
            let mut value = None;
            for override_ in content.parse_terminated::<_, Token![,]>(BinaryOverride::parse)? {
                match override_ {
                    BinaryOverride::Key { type_ } => key = Some(type_),
                    BinaryOverride::Value { type_ } => value = Some(type_),
                }
            }

            let key = key.ok_or_else(|| Error::new(name.span(), "missing `key` configuration"))?;
            let value =
                value.ok_or_else(|| Error::new(name.span(), "missing `value` configuration"))?;

            Ok(FieldOverride::Map { key, value })
        } else {
            Err(Error::new(
                name.span(),
                "expected `default`, `into`, `list`, or `set`",
            ))
        }
    }
}

enum UnaryOverride {
    Item { type_: TokenStream },
}

impl Parse for UnaryOverride {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let name = input.parse::<Ident>()?;
        if name == "item" {
            let content;
            parenthesized!(content in input);

            let mut type_ = None;
            for override_ in
                content.parse_terminated::<_, Token![,]>(CollectionTypeOverride::parse)?
            {
                match override_ {
                    CollectionTypeOverride::Type { type_: t } => type_ = Some(t),
                }
            }

            let type_ =
                type_.ok_or_else(|| Error::new(name.span(), "missing `type` configuration"))?;

            Ok(UnaryOverride::Item { type_ })
        } else {
            Err(Error::new(name.span(), "expected `item`"))
        }
    }
}

enum CollectionTypeOverride {
    Type { type_: TokenStream },
}

impl Parse for CollectionTypeOverride {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let name = input.call(Ident::parse_any)?;
        if name == "type" {
            input.parse::<Token![=]>()?;
            let type_ = input.parse::<Type>()?;

            Ok(CollectionTypeOverride::Type {
                type_: type_.to_token_stream(),
            })
        } else {
            Err(Error::new(name.span(), "expected `type`"))
        }
    }
}

enum BinaryOverride {
    Key { type_: TokenStream },
    Value { type_: TokenStream },
}

impl Parse for BinaryOverride {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let name = input.parse::<Ident>()?;
        if name == "key" || name == "value" {
            let content;
            parenthesized!(content in input);

            let mut type_ = None;
            for override_ in
                content.parse_terminated::<_, Token![,]>(CollectionTypeOverride::parse)?
            {
                match override_ {
                    CollectionTypeOverride::Type { type_: t } => type_ = Some(t),
                }
            }

            let type_ =
                type_.ok_or_else(|| Error::new(name.span(), "missing `type` configuration"))?;

            if name == "key" {
                Ok(BinaryOverride::Key { type_ })
            } else {
                Ok(BinaryOverride::Value { type_ })
            }
        } else {
            Err(Error::new(name.span(), "expected `key` or `value`"))
        }
    }
}
