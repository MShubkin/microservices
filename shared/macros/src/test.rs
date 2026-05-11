use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{spanned::Spanned, *};

const TESTING: &str = "testing";
const TEST_HARNESS: &str = "TestHarness";
const INITIALIZE: &str = "initialize";
const INITIALIZE_WITH: &str = "initialize_with";
const INITIALIZE_WITH_ARG: &str = "with_arg";

enum InitializeKind {
    Initialize,
    InitializeWith,
}

impl ToTokens for InitializeKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = match self {
            InitializeKind::Initialize => format_ident!("{INITIALIZE}"),
            InitializeKind::InitializeWith => format_ident!("{INITIALIZE_WITH}"),
        };
        ident.to_tokens(tokens);
    }
}

macro_rules! try_ {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return err.to_compile_error(),
        }
    };
}

fn arg_with_type(fn_arg: &FnArg) -> Result<&PatType> {
    match fn_arg {
        FnArg::Receiver(slf) => {
            Err(Error::new(slf.span(), "`self` is unexpected here"))
        }
        FnArg::Typed(pat_type) => Ok(pat_type),
    }
}

fn arg_with_type_mut(fn_arg: &mut FnArg) -> Result<&mut PatType> {
    match fn_arg {
        FnArg::Receiver(slf) => {
            Err(Error::new(slf.span(), "`self` is unexpected here"))
        }
        FnArg::Typed(pat_type) => Ok(pat_type),
    }
}

/// Returns full path to harness initialization function.
fn initialize(initialize: InitializeKind) -> TokenStream {
    let testing = format_ident!("{}", TESTING);
    let test_harness = format_ident!("{}", TEST_HARNESS);
    quote!(::#testing::#test_harness::#initialize)
}

/// Predicate that attribute represents argument to pass to initialize_with fn
fn is_initialize_with_arg(attr: &Attribute) -> bool {
    attr.path().is_ident(INITIALIZE_WITH_ARG)
}

fn arg_name(i: usize, pat: &Pat) -> TokenStream {
    if let Pat::Ident(PatIdent { ident, .. }) = pat {
        ident.to_token_stream()
    } else {
        format_ident!("__arg_{i}").to_token_stream()
    }
}

fn arg_to_name((i, fn_arg): (usize, &FnArg)) -> TokenStream {
    let PatType { pat, .. } = try_!(arg_with_type(fn_arg));
    arg_name(i, pat.as_ref())
}

fn arg_to_call_arg((i, fn_arg): (usize, &FnArg)) -> TokenStream {
    let PatType { pat, ty, .. } = try_!(arg_with_type(fn_arg));
    let arg_name = arg_name(i, pat.as_ref());
    if let Type::Reference(_) = ty.as_ref() {
        quote!(&#arg_name)
    } else {
        arg_name
    }
}

fn initialize_call(expr: Option<Expr>) -> TokenStream {
    if let Some(expr) = expr {
        let initialize = initialize(InitializeKind::InitializeWith);
        quote!(#initialize(#expr))
    } else {
        let initialize = initialize(InitializeKind::Initialize);
        quote!(#initialize())
    }
}

fn get_objs_with_args(fn_arg: &mut FnArg) -> Result<Option<Expr>> {
    let PatType { attrs, .. } = arg_with_type_mut(fn_arg)?;
    let (with_arg_attrs, _attrs) =
        std::mem::take(attrs).into_iter().partition(is_initialize_with_arg);
    *attrs = _attrs;
    let expr =
        with_arg_attrs.first().map(Attribute::parse_args::<Expr>).transpose()?;
    Ok(expr)
}

pub(super) fn run(_attr: TokenStream, mut test_func: ItemFn) -> TokenStream {
    let to_initialize = try_!(test_func
        .sig
        .inputs
        .iter_mut()
        .map(get_objs_with_args)
        .collect::<Result<Vec<_>>>());
    let ItemFn {
        sig:
            Signature {
                ident: name,
                inputs,
                output,
                ..
            },
        ..
    } = &test_func;
    let arg_names = inputs.iter().enumerate().map(arg_to_name);
    let call_args = inputs.iter().enumerate().map(arg_to_call_arg);
    let unwrap_or_pass = if matches!(output, ReturnType::Default) {
        quote!(.unwrap())
    } else {
        quote!(?)
    };
    let initialize = to_initialize.into_iter().map(initialize_call);
    quote!(
        #[tokio::test]
        async fn #name() #output {
            #test_func

            let (#(#arg_names,)*) = (#(#initialize.await #unwrap_or_pass,)*);

            #name(#(#call_args),*).await
        }
    )
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::parse_quote;

    use super::*;

    #[test]
    fn single_arg() {
        let test_func = parse_quote!(
            async fn foo(arg: Bar) {
                run_using_bar(&arg)
            }
        );
        let actual = super::run(quote!(), test_func);
        let initialize = super::initialize(InitializeKind::Initialize);
        let expected = quote!(
            #[tokio::test]
            async fn foo() {
                async fn foo(arg: Bar) {
                    run_using_bar(&arg)
                }

                let (arg,) = (#initialize().await.unwrap(),);

                foo(arg).await
            }
        );

        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn return_result() {
        let test_func = parse_quote!(
            async fn foo(arg: Bar) -> Result<(), Error> {
                run_using_bar(&arg)
            }
        );
        let actual = super::run(quote!(), test_func);
        let initialize = super::initialize(InitializeKind::Initialize);
        let expected = quote!(
            #[tokio::test]
            async fn foo() -> Result<(), Error> {
                async fn foo(arg: Bar) -> Result<(), Error> {
                    run_using_bar(&arg)
                }

                let (arg,) = (#initialize().await?,);

                foo(arg).await
            }
        );

        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn type_ref() {
        let test_func = parse_quote!(
            async fn foo(arg: &Bar) {
                run_using_bar(arg)
            }
        );
        let actual = super::run(quote!(), test_func);
        let initialize = super::initialize(InitializeKind::Initialize);
        let expected = quote!(
            #[tokio::test]
            async fn foo() {
                async fn foo(arg: &Bar) {
                    run_using_bar(arg)
                }

                let (arg,) = (#initialize().await.unwrap(),);

                foo(&arg).await
            }
        );

        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn with_args() {
        let test_func = parse_quote!(
            async fn foo(#[with_arg(quux(13 + 29))] arg: Bar) {
                run_using_bar(&arg)
            }
        );
        let actual = super::run(quote!(), test_func);
        let initialize_with = super::initialize(InitializeKind::InitializeWith);
        let expected = quote!(
            #[tokio::test]
            async fn foo() {
                async fn foo(arg: Bar) {
                    run_using_bar(&arg)
                }

                let (arg,) = (#initialize_with(quux(13 + 29)).await.unwrap(),);

                foo(arg).await
            }
        );

        assert_eq!(expected.to_string(), actual.to_string());
    }
}
