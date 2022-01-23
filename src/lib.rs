use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::token::{Comma, Paren};
use syn::{
    parenthesized, parse_macro_input, Data, DeriveInput, Error, Field, Fields, FieldsNamed, Ident,
    Token, Visibility,
};

#[proc_macro_derive(StagedBuilder, attributes(builder))]
pub fn derive_staged_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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
    let final_stage = final_stage(&input, &fields);

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
    let ty = &field.field.ty;

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

    let struct_docs = format!("The `{name}` stage builder for [`{}`]", input.ident);
    let setter_docs = format!("Sets the `{name}` field.");

    quote! {
        #[doc = #struct_docs]
        #vis struct #builder_name {
            #(#existing_names: #existing_types,)*
        }

        impl #builder_name {
            #[doc = #setter_docs]
            #[inline]
            pub fn #name(self, #name: #ty) -> #next_builder {
                #next_builder {
                    #(#existing_names: self.#existing_names,)*
                    #name,
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

fn final_stage(input: &DeriveInput, fields: &[ResolvedField<'_>]) -> TokenStream {
    let vis = stage_vis(&input.vis);
    let builder_name = final_name();
    let struct_name = &input.ident;
    let names = fields
        .iter()
        .map(|f| f.field.ident.as_ref().unwrap())
        .collect::<Vec<_>>();
    let types = fields.iter().map(|f| &f.field.ty).collect::<Vec<_>>();

    let setters = fields.iter().map(final_stage_setter);

    let struct_docs = format!("The final stage builder for [`{struct_name}`]");
    let build_docs = format!("Consumes the builder, returning a [`{struct_name}`].");

    quote! {
        #[doc = #struct_docs]
        #vis struct #builder_name {
            #(#names: #types,)*
        }

        impl #builder_name {
            #(#setters)*

            #[doc = #build_docs]
            #[inline]
            pub fn build(self) -> #struct_name {
                #struct_name {
                    #(#names: self.#names,)*
                }
            }
        }

        impl From<#struct_name> for #builder_name {
            #[inline]
            fn from(s: #struct_name) -> Self {
                #builder_name {
                    #(#names: s.#names,)*
                }
            }
        }
    }
}

fn final_stage_setter(field: &ResolvedField<'_>) -> TokenStream {
    let name = field.field.ident.as_ref().unwrap();
    let ty = &field.field.ty;

    let docs = format!("Sets the `{name}` field.");

    quote! {
        #[doc = #docs]
        #[inline]
        pub fn #name(mut self, #name: #ty) -> Self {
            self.#name = #name;
            self
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

struct ResolvedField<'a> {
    field: &'a Field,
    default: Option<TokenStream>,
}

impl<'a> ResolvedField<'a> {
    fn new(field: &'a Field) -> Result<ResolvedField<'a>, Error> {
        let mut resolved = ResolvedField {
            field,
            default: None,
        };

        for attr in &field.attrs {
            if !attr.path.is_ident("builder") {
                continue;
            }

            let overrides = attr.parse_args_with(|p: ParseStream<'_>| {
                p.parse_terminated::<_, Comma>(Override::parse)
            })?;

            for override_ in overrides {
                match override_ {
                    Override::Default { expr } => {
                        resolved.default =
                            Some(expr.unwrap_or(quote!(std::default::Default::default())));
                    }
                }
            }
        }

        Ok(resolved)
    }
}

enum Override {
    Default { expr: Option<TokenStream> },
}

impl Parse for Override {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let name = input.lookahead1();
        if name.peek(Token![default]) {
            input.parse::<Token![default]>()?;
            let expr = if input.peek(Paren) {
                let content;
                parenthesized!(content in input);
                let expr = content.parse::<TokenStream>()?;
                if !content.is_empty() {
                    return Err(Error::new(content.span(), "unexpected trailing data"));
                }

                Some(expr)
            } else {
                None
            };

            Ok(Override::Default { expr })
        } else {
            Err(name.error())
        }
    }
}
