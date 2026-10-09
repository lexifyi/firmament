use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);

    if func.sig.ident != "main" {
        abort!(func.sig.ident.span(), "must be called `main`");
    }

    if func.sig.asyncness.is_none() {
        abort!(func.sig.ident.span(), "main function must be async");
    }

    let ItemFn {
        attrs,
        vis,
        sig,
        block,
        ..
    } = &func;

    let main_fn = match sig.output {
        syn::ReturnType::Default => quote! {
            async fn __main() -> firmament::app::Result {
                main().await;

                Ok(())
            }
        },

        _ => quote! {
            let __main = main;
        },
    };

    let expanded = quote! {
        #vis fn main() -> std::process::ExitCode {
            #(#attrs)*
            #sig
            #block

            let module_path = module_path!();
            let crate_name = module_path.split_once("::").map_or(module_path, |x| x.0);

            #main_fn

            match firmament::app::__run(crate_name, __main) {
                Ok(_) => std::process::ExitCode::SUCCESS,

                Err(err) => {
                    log::error!("{err}");

                    std::process::ExitCode::FAILURE
                }
            }
        }
    };

    TokenStream::from(expanded)
}
