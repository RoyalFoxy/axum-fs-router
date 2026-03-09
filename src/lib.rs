use crate::segmets::Routes;
use quote::quote;

mod segmets;

const ROUTES_FOLDER: &str = "./src/routes";

fn traverse_routes_wrapper() -> eyre::Result<proc_macro2::TokenStream> {
    let routes = Routes::new(ROUTES_FOLDER)?;

    Ok(quote! {
        #[path = "./routes"]
        pub mod __generated_routes { #routes }
    })
}

#[proc_macro]
pub fn traverse_routes(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match traverse_routes_wrapper() {
        Ok(tokens) => tokens,
        Err(error) => {
            let error = format!("{error:?}");
            quote! { compiler_error!(#error) }
        }
    }
    .into()
}

fn router_wrapper() -> eyre::Result<proc_macro2::TokenStream> {
    let router = Routes::new(ROUTES_FOLDER)?.to_router();

    Ok(quote! {
       {
            use __generated_routes::*;

            #router
       }
    })
}

#[proc_macro]
pub fn router(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match router_wrapper() {
        Ok(tokens) => tokens,
        Err(error) => {
            let error = format!("{error:?}");
            quote! { compiler_error!(#error) }
        }
    }
    .into()
}
