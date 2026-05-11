use nom::bytes::complete::{take_while, take_while1};
use nom::multi::separated_list1;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::parse::Parse;

use quote::{quote, ToTokens};
use syn::parse::{ParseStream, Result};
use syn::{parse_quote, parse_str, Error, Expr, Ident, LitStr, Token};

const CONFORMED_RELATION_FUNC: &str =
    "shared_essential::application::message::numeral_relation::get_conformed_numeric";
const CONTROLLED_RELATION_FUNC: &str =
    "shared_essential::application::message::numeral_relation::get_controlled_numeric";

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    combinator::recognize,
    multi::many0,
    sequence::delimited,
    IResult,
};

enum NumericFormatError {
    InvalidFormatStr(String),
    NotFoundNumericParam,
    SeveralNumericParams,
}

#[derive(Debug)]
enum FormatPart<'a> {
    Text(&'a str),
    Placeholder(&'a str),
    NumericPlaceholder(NumericPlaceholder<'a>),
    NumericParam(&'a str),
}

#[derive(Debug)]
enum NumericPlaceholder<'a> {
    Controlled {
        singular: &'a str,
        within_2_and_4_in_the_end: &'a str,
        default: &'a str,
    },
    Conformed {
        singular: &'a str,
        plural: &'a str,
    },
}

#[derive(Debug)]
struct NumericFormatArgs {
    format_str: String,
    params: Vec<Expr>,
    numeric_param: Option<Expr>,
}

impl Parse for NumericFormatArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let format_str = input.parse::<LitStr>()?.value();

        if !input.peek(Token![,]) {
            return Ok(Self {
                format_str,
                numeric_param: None,
                params: Vec::default(),
            });
        }

        input.parse::<Token![,]>()?;

        let mut params = Vec::new();
        let mut numeric_param = None;

        loop {
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![@]) {
                let _at = input.parse::<Token![@]>()?;
                if numeric_param.is_some() {
                    return Err(NumericFormatError::SeveralNumericParams.into());
                } else {
                    let expr: Expr = input.parse()?;
                    // Сам параметр не пушим в параметры
                    numeric_param = Some(expr);
                }
            } else {
                let param: Expr = input.parse()?;
                params.push(param);
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                break;
            }
        }

        Ok(Self {
            format_str,
            params,
            numeric_param,
        })
    }
}

pub fn numeric_format_inner(input: TokenStream) -> Result<TokenStream> {
    let NumericFormatArgs {
        format_str,
        mut params,
        numeric_param,
    } = syn::parse(input)?;

    let (_, format_parts) = parse_format_parts(&format_str)
        .map_err(|_| NumericFormatError::InvalidFormatStr(format_str.clone()))?;
    let mut numeric_vars_count = 0;

    let numeric_param = match numeric_param {
        Some(param) => {
            if format_parts.iter().any(|p| matches!(p, FormatPart::NumericParam(_)))
            {
                return Err(NumericFormatError::SeveralNumericParams.into());
            }
            param
        }
        None => {
            let mut numeric_params = format_parts.iter().filter_map(|p| {
                if let FormatPart::NumericParam(np) = p {
                    Some(np)
                } else {
                    None
                }
            });
            if let Some(base_numeric) = numeric_params.next() {
                if numeric_params
                    .any(|another_numeric| another_numeric != base_numeric)
                {
                    return Err(NumericFormatError::SeveralNumericParams.into());
                } else {
                    parse_str(base_numeric).unwrap()
                }
            } else {
                return Err(NumericFormatError::NotFoundNumericParam.into());
            }
        }
    };

    let mut format_string = String::new();
    for part in format_parts {
        match part {
            FormatPart::Text(text) | FormatPart::Placeholder(text) => {
                format_string.push_str(text);
            }
            FormatPart::NumericParam(param) => {
                format_string.push('{');
                format_string.push_str(param);
                format_string.push('}');
            }
            FormatPart::NumericPlaceholder(placeholder) => {
                let var_name = format!("count_{numeric_vars_count}");
                let var_ident = Ident::new(&var_name, Span::call_site());
                numeric_vars_count += 1;

                let numeric_relation = match placeholder {
                    NumericPlaceholder::Controlled {
                        singular,
                        within_2_and_4_in_the_end,
                        default,
                    } => {
                        let func_path: proc_macro2::TokenStream =
                            CONTROLLED_RELATION_FUNC.parse().unwrap();

                        quote! {
                            #func_path(
                                #numeric_param,
                                #singular,
                                #within_2_and_4_in_the_end,
                                #default
                            )
                        }
                    }
                    NumericPlaceholder::Conformed { singular, plural } => {
                        let func_path: proc_macro2::TokenStream =
                            CONFORMED_RELATION_FUNC.parse().unwrap();

                        quote! {
                            #func_path(
                                #numeric_param,
                                #singular,
                                #plural
                            )
                        }
                    }
                };

                let format_param: Expr = parse_quote!(
                    #var_ident = #numeric_relation
                );
                params.push(format_param);

                format_string.push('{');
                format_string.push_str(&var_name);
                format_string.push('}');
            }
        }
    }

    let params =
        params.into_iter().map(|p| p.to_token_stream()).collect::<Vec<_>>();

    Ok(quote!(format!(#format_string, #(#params),*)).into())
}

fn parse_format_parts<'a>(input: &'a str) -> IResult<&'a str, Vec<FormatPart<'a>>> {
    many0(alt((
        parse_numeric_placeholders,
        parse_numeric_param,
        parse_regular_placeholder,
        parse_text,
    )))(input)
}

fn parse_text<'a>(input: &'a str) -> IResult<&'a str, FormatPart<'a>> {
    let (remaining, text) = take_while1(|c| c != '{')(input)?;
    Ok((remaining, FormatPart::Text(text)))
}

fn parse_regular_placeholder<'a>(input: &'a str) -> IResult<&'a str, FormatPart<'a>> {
    let (remaining, placeholder) =
        recognize(delimited(tag("{"), take_until("}"), tag("}")))(input)?;

    Ok((remaining, FormatPart::Placeholder(placeholder)))
}

fn parse_numeric_param<'a>(input: &'a str) -> IResult<&'a str, FormatPart<'a>> {
    let (remaining, param) =
        delimited(tag("{@"), take_while(|c: char| c != '}'), tag("}"))(input)?;

    Ok((remaining, FormatPart::NumericParam(param)))
}

fn parse_numeric_placeholders<'a>(input: &'a str) -> IResult<&'a str, FormatPart<'a>> {
    let (remaining, forms) = delimited(
        tag("{@"),
        separated_list1(tag("|"), take_while(|c: char| c != '|' && c != '}')),
        tag("}"),
    )(input)?;

    let numeric_placeholder = match forms.as_slice() {
        [singular, plural] => {
            Ok(NumericPlaceholder::Conformed { singular, plural })
        }
        [singular, within_2_and_4_in_the_end, default] => {
            Ok(NumericPlaceholder::Controlled {
                singular,
                within_2_and_4_in_the_end,
                default,
            })
        }
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::LengthValue,
        ))),
    }?;

    Ok((remaining, FormatPart::NumericPlaceholder(numeric_placeholder)))
}

impl From<NumericFormatError> for Error {
    fn from(value: NumericFormatError) -> Self {
        match value {
            NumericFormatError::InvalidFormatStr(str) => Error::new(
                Span::call_site(),
                format!("Невалидная форматирующая строка `{str}`"),
            ),
            NumericFormatError::NotFoundNumericParam => Error::new(
                Span::call_site(),
                String::from("Параметр с числительным не был передан"),
            ),
            NumericFormatError::SeveralNumericParams => Error::new(
                Span::call_site(),
                String::from("Параметр с числительным был передан несколько раз"),
            ),
        }
    }
}
