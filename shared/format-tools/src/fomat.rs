use proc_macro::TokenStream;
use syn::parse::Parse;

use quote::quote;
use syn::parse::{ParseStream, Result};
use syn::{LitStr, Token};

#[derive(Debug)]
struct FomatArgs {
    format_signs: (String, String),
    str: String,
    params: proc_macro2::TokenStream,
}

impl Parse for FomatArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let open = input.parse::<LitStr>()?.value();
        input.parse::<Token![,]>()?;
        let close = input.parse::<LitStr>()?.value();
        input.parse::<Token![,]>()?;

        let str = input.parse::<LitStr>()?.value();
        input.parse::<Token![,]>()?;

        let params = input.parse::<proc_macro2::TokenStream>()?;

        Ok(FomatArgs {
            format_signs: (open, close),
            str,
            params,
        })
    }
}

pub(crate) fn fomat_inner(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as FomatArgs);

    let (open, close) = args.format_signs;
    let str = args.str;
    let params = args.params;

    let str_chars = str.chars().collect::<Vec<_>>();
    let (open_chars, close_chars) =
        (open.chars().collect::<Vec<_>>(), close.chars().collect::<Vec<_>>());

    let mut format_str = String::with_capacity(str.len());
    let mut cursor = 0;

    // Следующий цикл переписывает строку , меняя паттерны указанные
    // как "отрывающие" и "закрывающие" на `{` и `}`, и уже существующие `{` и `}` на
    // `{{` и `}}` чтобы из входной строки сделать строку для `format!` макроса
    while let Some(&c) = str_chars.get(cursor) {
        match c {
            '{' => {
                format_str.push_str("{{");
                cursor += 1
            }
            '}' => {
                format_str.push_str("}}");
                cursor += 1
            }
            _ if str_chars[cursor..].starts_with(&open_chars) => {
                format_str.push('{');
                cursor += open_chars.len();
            }
            _ if str_chars[cursor..].starts_with(&close_chars) => {
                format_str.push('}');
                cursor += close_chars.len();
            }
            n => {
                format_str.push(n);
                cursor += 1
            }
        };
    }

    let output = quote! {
        format!(#format_str, #params)
    };

    TokenStream::from(output)
}
