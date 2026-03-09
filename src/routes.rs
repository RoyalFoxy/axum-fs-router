use eyre::{Result, eyre};
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::quote;
use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};
use syn::Ident;

#[derive(Debug)]
pub struct Routes {
    inner: Vec<Segment>,
}

#[derive(Debug)]
pub enum Segment {
    Folder {
        name: String,
        path: String,
        sub: Routes,
    },
    Handler {
        name: String,
    },
}

impl Routes {
    pub fn new(folder: impl Into<PathBuf>) -> Result<Self> {
        let mut inner = Vec::new();

        let entries =
            fs::read_dir(folder.into())?.collect::<Result<Vec<DirEntry>, std::io::Error>>()?;

        for entry in entries {
            let file_type = entry.file_type()?;

            let file_path = entry.file_name().into_string().map_err(|_| {
                eyre!(
                    "Invalid filename {file}",
                    file = entry.file_name().display()
                )
            })?;

            if file_type.is_dir() {
                let name = String::from(file_path.trim_matches(|char| char == '{' || char == '}'));

                let sub = Self::new(entry.path())?;

                inner.push(Segment::Folder {
                    name,
                    path: file_path,
                    sub,
                });
            } else {
                let name = file_path.replace(".rs", "");

                match name.as_str() {
                    "any" | "connect" | "delete" | "get" | "head" | "options" | "patch"
                    | "post" | "put" | "trace" => (),

                    _ => continue,
                }

                inner.push(Segment::Handler { name });
            }
        }

        Ok(Self { inner })
    }
}

impl ToTokens for Segment {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Segment::Folder { name, sub, path } => {
                let name = Ident::new(name, Span::call_site());

                tokens.extend(quote! {
                    #[path = #path]
                    pub mod #name { #sub }
                });
            }
            Segment::Handler { name } => {
                let name = Ident::new(name, Span::call_site());

                tokens.extend(quote! { pub mod #name; });
            }
        }
    }
}

impl ToTokens for Routes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let segments = &self.inner;
        tokens.extend(quote! { #(#segments)* });
    }
}

impl Segment {
    fn to_handler(&self) -> TokenStream {
        match self {
            Segment::Folder { name, path, sub } => {
                let relative_route_path = format!("/{path}");
                let sub_router = sub.to_router();
                let module = Ident::new(name, Span::call_site());

                quote! { .nest(#relative_route_path, {
                    use #module::*;
                    #sub_router
                }) }
            }

            Segment::Handler { name } => {
                let method = Ident::new(name, Span::call_site());

                quote! { .route("/", ::axum::routing::#method(#method::handler)) }
            }
        }
    }
}

impl Routes {
    pub fn to_router(&self) -> TokenStream {
        let streams = self
            .inner
            .iter()
            .map(Segment::to_handler)
            .collect::<Vec<TokenStream>>();

        quote! { ::axum::routing::Router::new()#(#streams)* }
    }
}
