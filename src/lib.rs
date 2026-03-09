use crate::segmets::Segments;
use quote::quote;
use std::sync::LazyLock;

mod segmets;

static ROUTES: LazyLock<Segments> =
    LazyLock::new(|| segmets::Segments::new("./src/routes").unwrap());

#[proc_macro]
pub fn traverse_routes(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let routes = &*ROUTES;

    quote! {
        #[path = "../routes"]
        pub mod __generated_routes { #routes }
    }
    .into()
}

#[proc_macro]
pub fn router(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let router = (*ROUTES).to_router();

    quote! {
       {
            use __generated_routes::*;

            #router
       }
    }
    .into()
}
