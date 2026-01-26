//! # maple_derive
//!
//! This crate provides a custom derive macro for implementing the `Node` trait in the `maple` game engine.
//!
//! ## Usage
//!
//! To use, annotate your struct like this:
//!
//! ```rust,ignore
//! use maple::engine::{
//!     Node,
//!     components::{NodeTransform, EventReceiver},
//!     context::Scene
//! };
//!
//! #[derive(Node)]
//! struct MyNode {
//!     #[transform]
//!     transform: NodeTransform,
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Derives the `Node` trait for a struct with fields for transform, children, and events.
///
/// ## Example
///
/// ```rust,ignore
/// use maple::{
///     Node,
///     components::{NodeTransform, EventReceiver},
///     context::Scene
/// };
///
/// #[derive(Node)]
/// struct MyNode {
///     #[transform]
///     transform: NodeTransform,
/// }
/// ```
#[proc_macro_derive(Node, attributes(transform))]
pub fn derive_node(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => &fields_named.named,
            _ => panic!("Node can only be derived for structs with named fields"),
        },
        _ => panic!("Node can only be derived for structs"),
    };

    let mut transform_field = None;

    for field in fields {
        for attr in &field.attrs {
            if attr.path().is_ident("transform") {
                transform_field = Some(field.ident.clone().unwrap());
            }
        }
    }

    if transform_field.is_none() {
        let mut errors = Vec::new();

        if transform_field.is_none() {
            errors.push(syn::Error::new_spanned(
                struct_name,
                "Missing field marked with #[transform]\n\
        Example:\n\
        #[transform]\n\
        transform: NodeTransform,",
            ));
        }

        let combined = errors
            .into_iter()
            .map(|e| e.to_compile_error())
            .collect::<proc_macro2::TokenStream>();

        return TokenStream::from(combined);
    }

    let transform = transform_field.unwrap();

    let expanded = quote! {

        impl ::maple::engine::nodes::Node for #struct_name
            {
            fn get_transform(&mut self) -> &mut ::maple::engine::components::NodeTransform {
                &mut self.#transform
            }

        }
    };

    TokenStream::from(expanded)
}
