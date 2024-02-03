use structmeta::{NameArgs, NameValue};
use syn::{Expr, Ident, Type};
pub struct FieldOverrides {
    pub default: Option<NameValue<Option<Expr>>>,
    pub into: bool,
    pub custom: Option<NameArgs<CustomOverrides>>,
    pub list: Option<NameArgs<SeqOverrides>>,
    pub set: Option<NameArgs<SeqOverrides>>,
    pub map: Option<NameArgs<MapOverrides>>,
    pub stage: Option<Ident>,
}
#[automatically_derived]
impl ::syn::parse::Parse for FieldOverrides {
    fn parse(input: ::syn::parse::ParseStream<'_>) -> ::syn::Result<Self> {
        let mut _value_2 = None;
        let mut _value_0 = None;
        let mut _value_1 = None;
        let mut _value_3 = None;
        let mut _value_5 = None;
        let mut _value_4 = None;
        let mut _value_6 = None;
        let mut is_next = false;
        let mut unnamed_index = 0;
        let mut named_used = false;
        while !input.is_empty() {
            if is_next {
                input.parse::<::syn::token::Comma>()?;
                if input.is_empty() {
                    break;
                }
            }
            is_next = true;
            if let Some((index, span)) = ::structmeta::helpers::try_parse_name(
                input,
                &["default", "into"],
                false,
                &["default", "stage"],
                false,
                &["custom", "list", "map", "set"],
                false,
                true,
                &(|_| true),
            )? {
                named_used = true;
                match index {
                    ::structmeta::helpers::NameIndex::Flag(Ok(0usize)) => {
                        if _value_0.is_some() {
                            return Err(::syn::Error::new(
                                span,
                                "parameter `default` specified more than once",
                            ));
                        }
                        _value_0 = Some(::structmeta::NameValue {
                            name_span: span,
                            value: None,
                        });
                    }
                    ::structmeta::helpers::NameIndex::Flag(Ok(1usize)) => {
                        if _value_1.is_some() {
                            return Err(::syn::Error::new(
                                span,
                                "parameter `into` specified more than once",
                            ));
                        }
                        _value_1 = Some(span);
                    }
                    ::structmeta::helpers::NameIndex::NameValue(Ok(0usize)) => {
                        if _value_0.is_some() {
                            return Err(::syn::Error::new(
                                span,
                                "parameter `default` specified more than once",
                            ));
                        }
                        _value_0 = Some(::structmeta::NameValue {
                            name_span: span,
                            value: Some(input.parse::<Expr>()?),
                        });
                    }
                    ::structmeta::helpers::NameIndex::NameValue(Ok(1usize)) => {
                        if _value_6.is_some() {
                            return Err(::syn::Error::new(
                                span,
                                "parameter `stage` specified more than once",
                            ));
                        }
                        _value_6 = Some(input.parse::<Ident>()?);
                    }
                    ::structmeta::helpers::NameIndex::NameArgs(Ok(0usize)) => {
                        if _value_2.is_some() {
                            return Err(::syn::Error::new(
                                span,
                                "parameter `custom` specified more than once",
                            ));
                        }
                        _value_2 = Some(structmeta::NameArgs {
                            name_span: span,
                            args: {
                                let content;
                                match ::syn::__private::parse_parens(&input) {
                                    ::syn::__private::Ok(parens) => {
                                        content = parens.content;
                                        parens.token
                                    }
                                    ::syn::__private::Err(error) => {
                                        return ::syn::__private::Err(error);
                                    }
                                };
                                content.parse::<CustomOverrides>()?
                            },
                        });
                    }
                    ::structmeta::helpers::NameIndex::NameArgs(Ok(1usize)) => {
                        if _value_3.is_some() {
                            return Err(::syn::Error::new(
                                span,
                                "parameter `list` specified more than once",
                            ));
                        }
                        _value_3 = Some(structmeta::NameArgs {
                            name_span: span,
                            args: {
                                let content;
                                match ::syn::__private::parse_parens(&input) {
                                    ::syn::__private::Ok(parens) => {
                                        content = parens.content;
                                        parens.token
                                    }
                                    ::syn::__private::Err(error) => {
                                        return ::syn::__private::Err(error);
                                    }
                                };
                                content.parse::<SeqOverrides>()?
                            },
                        });
                    }
                    ::structmeta::helpers::NameIndex::NameArgs(Ok(2usize)) => {
                        if _value_5.is_some() {
                            return Err(::syn::Error::new(
                                span,
                                "parameter `map` specified more than once",
                            ));
                        }
                        _value_5 = Some(structmeta::NameArgs {
                            name_span: span,
                            args: {
                                let content;
                                match ::syn::__private::parse_parens(&input) {
                                    ::syn::__private::Ok(parens) => {
                                        content = parens.content;
                                        parens.token
                                    }
                                    ::syn::__private::Err(error) => {
                                        return ::syn::__private::Err(error);
                                    }
                                };
                                content.parse::<MapOverrides>()?
                            },
                        });
                    }
                    ::structmeta::helpers::NameIndex::NameArgs(Ok(3usize)) => {
                        if _value_4.is_some() {
                            return Err(::syn::Error::new(
                                span,
                                "parameter `set` specified more than once",
                            ));
                        }
                        _value_4 = Some(structmeta::NameArgs {
                            name_span: span,
                            args: {
                                let content;
                                match ::syn::__private::parse_parens(&input) {
                                    ::syn::__private::Ok(parens) => {
                                        content = parens.content;
                                        parens.token
                                    }
                                    ::syn::__private::Err(error) => {
                                        return ::syn::__private::Err(error);
                                    }
                                };
                                content.parse::<SeqOverrides>()?
                            },
                        });
                    }
                    _ => panic!("internal error: entered unreachable code"),
                }
            } else {
                return Err(input.error("cannot use unnamed parameter"));
            }
        }
        Ok(Self {
            default: _value_0,
            into: _value_1.is_some(),
            custom: _value_2,
            list: _value_3,
            set: _value_4,
            map: _value_5,
            stage: _value_6,
        })
    }
}
#[automatically_derived]
impl ::core::default::Default for FieldOverrides {
    #[inline]
    fn default() -> FieldOverrides {
        FieldOverrides {
            default: ::core::default::Default::default(),
            into: ::core::default::Default::default(),
            custom: ::core::default::Default::default(),
            list: ::core::default::Default::default(),
            set: ::core::default::Default::default(),
            map: ::core::default::Default::default(),
            stage: ::core::default::Default::default(),
        }
    }
}
pub struct CustomOverrides {
    pub type_: Type,
    pub convert: Expr,
}
#[automatically_derived]
impl ::syn::parse::Parse for CustomOverrides {
    fn parse(input: ::syn::parse::ParseStream<'_>) -> ::syn::Result<Self> {
        let mut _value_1 = None;
        let mut _value_0 = None;
        let mut is_next = false;
        let mut unnamed_index = 0;
        let mut named_used = false;
        while !input.is_empty() {
            if is_next {
                input.parse::<::syn::token::Comma>()?;
                if input.is_empty() {
                    break;
                }
            }
            is_next = true;
            if let Some((index, span)) = ::structmeta::helpers::try_parse_name(
                input,
                &[],
                false,
                &["convert", "type"],
                false,
                &[],
                false,
                true,
                &(|_| true),
            )? {
                named_used = true;
                match index {
                    ::structmeta::helpers::NameIndex::NameValue(Ok(0usize)) => {
                        if _value_1.is_some() {
                            return Err(::syn::Error::new(
                                span,
                                "parameter `convert` specified more than once",
                            ));
                        }
                        _value_1 = Some(input.parse::<Expr>()?);
                    }
                    ::structmeta::helpers::NameIndex::NameValue(Ok(1usize)) => {
                        if _value_0.is_some() {
                            return Err(::syn::Error::new(
                                span,
                                "parameter `type` specified more than once",
                            ));
                        }
                        _value_0 = Some(input.parse::<Type>()?);
                    }
                    _ => panic!("internal error: entered unreachable code"),
                }
            } else {
                return Err(input.error("cannot use unnamed parameter"));
            }
        }
        Ok(Self {
            type_: _value_0.ok_or_else(|| {
                ::syn::Error::new(
                    ::proc_macro2::Span::call_site(),
                    "missing argument `type = ...`",
                )
            })?,
            convert: _value_1.ok_or_else(|| {
                ::syn::Error::new(
                    ::proc_macro2::Span::call_site(),
                    "missing argument `convert = ...`",
                )
            })?,
        })
    }
}
pub struct SeqOverrides {
    pub item: NameArgs<ParamOverrides>,
}
#[automatically_derived]
impl ::syn::parse::Parse for SeqOverrides {
    fn parse(input: ::syn::parse::ParseStream<'_>) -> ::syn::Result<Self> {
        let mut _value_0 = None;
        let mut is_next = false;
        let mut unnamed_index = 0;
        let mut named_used = false;
        while !input.is_empty() {
            if is_next {
                input.parse::<::syn::token::Comma>()?;
                if input.is_empty() {
                    break;
                }
            }
            is_next = true;
            if let Some((index, span)) = ::structmeta::helpers::try_parse_name(
                input,
                &[],
                false,
                &[],
                false,
                &["item"],
                false,
                true,
                &(|_| true),
            )? {
                named_used = true;
                match index {
                    ::structmeta::helpers::NameIndex::NameArgs(Ok(0usize)) => {
                        if _value_0.is_some() {
                            return Err(::syn::Error::new(
                                span,
                                "parameter `item` specified more than once",
                            ));
                        }
                        _value_0 = Some(structmeta::NameArgs {
                            name_span: span,
                            args: {
                                let content;
                                match ::syn::__private::parse_parens(&input) {
                                    ::syn::__private::Ok(parens) => {
                                        content = parens.content;
                                        parens.token
                                    }
                                    ::syn::__private::Err(error) => {
                                        return ::syn::__private::Err(error);
                                    }
                                };
                                content.parse::<ParamOverrides>()?
                            },
                        });
                    }
                    _ => panic!("internal error: entered unreachable code"),
                }
            } else {
                return Err(input.error("cannot use unnamed parameter"));
            }
        }
        Ok(Self {
            item: _value_0.ok_or_else(|| {
                ::syn::Error::new(
                    ::proc_macro2::Span::call_site(),
                    "missing argument `item(...)`",
                )
            })?,
        })
    }
}
pub struct ParamOverrides {
    pub type_: Option<Type>,
    pub into: bool,
    pub custom: Option<NameArgs<CustomOverrides>>,
}
#[automatically_derived]
impl ::syn::parse::Parse for ParamOverrides {
    fn parse(input: ::syn::parse::ParseStream<'_>) -> ::syn::Result<Self> {
        let mut _value_2 = None;
        let mut _value_1 = None;
        let mut _value_0 = None;
        let mut is_next = false;
        let mut unnamed_index = 0;
        let mut named_used = false;
        while !input.is_empty() {
            if is_next {
                input.parse::<::syn::token::Comma>()?;
                if input.is_empty() {
                    break;
                }
            }
            is_next = true;
            if let Some((index, span)) = ::structmeta::helpers::try_parse_name(
                input,
                &["into"],
                false,
                &["type"],
                false,
                &["custom"],
                false,
                true,
                &(|_| true),
            )? {
                named_used = true;
                match index {
                    ::structmeta::helpers::NameIndex::Flag(Ok(0usize)) => {
                        if _value_1.is_some() {
                            return Err(::syn::Error::new(
                                span,
                                "parameter `into` specified more than once",
                            ));
                        }
                        _value_1 = Some(span);
                    }
                    ::structmeta::helpers::NameIndex::NameValue(Ok(0usize)) => {
                        if _value_0.is_some() {
                            return Err(::syn::Error::new(
                                span,
                                "parameter `type` specified more than once",
                            ));
                        }
                        _value_0 = Some(input.parse::<Type>()?);
                    }
                    ::structmeta::helpers::NameIndex::NameArgs(Ok(0usize)) => {
                        if _value_2.is_some() {
                            return Err(::syn::Error::new(
                                span,
                                "parameter `custom` specified more than once",
                            ));
                        }
                        _value_2 = Some(structmeta::NameArgs {
                            name_span: span,
                            args: {
                                let content;
                                match ::syn::__private::parse_parens(&input) {
                                    ::syn::__private::Ok(parens) => {
                                        content = parens.content;
                                        parens.token
                                    }
                                    ::syn::__private::Err(error) => {
                                        return ::syn::__private::Err(error);
                                    }
                                };
                                content.parse::<CustomOverrides>()?
                            },
                        });
                    }
                    _ => panic!("internal error: entered unreachable code"),
                }
            } else {
                return Err(input.error("cannot use unnamed parameter"));
            }
        }
        Ok(Self {
            type_: _value_0,
            into: _value_1.is_some(),
            custom: _value_2,
        })
    }
}
pub struct MapOverrides {
    pub key: NameArgs<ParamOverrides>,
    pub value: NameArgs<ParamOverrides>,
}
#[automatically_derived]
impl ::syn::parse::Parse for MapOverrides {
    fn parse(input: ::syn::parse::ParseStream<'_>) -> ::syn::Result<Self> {
        let mut _value_0 = None;
        let mut _value_1 = None;
        let mut is_next = false;
        let mut unnamed_index = 0;
        let mut named_used = false;
        while !input.is_empty() {
            if is_next {
                input.parse::<::syn::token::Comma>()?;
                if input.is_empty() {
                    break;
                }
            }
            is_next = true;
            if let Some((index, span)) = ::structmeta::helpers::try_parse_name(
                input,
                &[],
                false,
                &[],
                false,
                &["key", "value"],
                false,
                true,
                &(|_| true),
            )? {
                named_used = true;
                match index {
                    ::structmeta::helpers::NameIndex::NameArgs(Ok(0usize)) => {
                        if _value_0.is_some() {
                            return Err(::syn::Error::new(
                                span,
                                "parameter `key` specified more than once",
                            ));
                        }
                        _value_0 = Some(structmeta::NameArgs {
                            name_span: span,
                            args: {
                                let content;
                                match ::syn::__private::parse_parens(&input) {
                                    ::syn::__private::Ok(parens) => {
                                        content = parens.content;
                                        parens.token
                                    }
                                    ::syn::__private::Err(error) => {
                                        return ::syn::__private::Err(error);
                                    }
                                };
                                content.parse::<ParamOverrides>()?
                            },
                        });
                    }
                    ::structmeta::helpers::NameIndex::NameArgs(Ok(1usize)) => {
                        if _value_1.is_some() {
                            return Err(::syn::Error::new(
                                span,
                                "parameter `value` specified more than once",
                            ));
                        }
                        _value_1 = Some(structmeta::NameArgs {
                            name_span: span,
                            args: {
                                let content;
                                match ::syn::__private::parse_parens(&input) {
                                    ::syn::__private::Ok(parens) => {
                                        content = parens.content;
                                        parens.token
                                    }
                                    ::syn::__private::Err(error) => {
                                        return ::syn::__private::Err(error);
                                    }
                                };
                                content.parse::<ParamOverrides>()?
                            },
                        });
                    }
                    _ => panic!("internal error: entered unreachable code"),
                }
            } else {
                return Err(input.error("cannot use unnamed parameter"));
            }
        }
        Ok(Self {
            key: _value_0.ok_or_else(|| {
                ::syn::Error::new(
                    ::proc_macro2::Span::call_site(),
                    "missing argument `key(...)`",
                )
            })?,
            value: _value_1.ok_or_else(|| {
                ::syn::Error::new(
                    ::proc_macro2::Span::call_site(),
                    "missing argument `value(...)`",
                )
            })?,
        })
    }
}
