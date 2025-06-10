//! # maple_derive
//!
//! This crate provides a custom derive macro for implementing the `Node` trait in the `maple` game engine.
//!
//! ## Usage
//!
//! To use, annotate your struct like this:
//!
//! ```rust
//! use maple::{
//!     Node,
//!     components::{NodeTransform, EventReceiver},
//!     context::Scene
//! };
//!
//! #[derive(Node, Clone)]
//! struct MyNode {
//!     #[transform]
//!     transform: NodeTransform,
//!
//!     #[children]
//!     children: Scene,
//!
//!     #[events]
//!     events: EventReceiver,
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Derives the `Node` trait for a struct with fields for transform, children, and events.
///
/// ## Example
///
/// ```rust
/// use maple::{
///     Node,
///     components::{NodeTransform, EventReceiver},
///     context::Scene
/// };
///
/// #[derive(Node, Clone)]
/// struct MyNode {
///     #[transform]
///     transform: NodeTransform,
///
///     #[children]
///     children: Scene,
///
///     #[events]
///     events: EventReceiver,
/// }
/// ```
#[proc_macro_derive(Node, attributes(transform, children, events))]
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
    let mut children_field = None;
    let mut events_field = None;

    for field in fields {
        for attr in &field.attrs {
            if attr.path().is_ident("transform") {
                transform_field = Some(field.ident.clone().unwrap());
            } else if attr.path().is_ident("children") {
                children_field = Some(field.ident.clone().unwrap());
            } else if attr.path().is_ident("events") {
                events_field = Some(field.ident.clone().unwrap());
            }
        }
    }

    if transform_field.is_none() || children_field.is_none() || events_field.is_none() {
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

        if children_field.is_none() {
            errors.push(syn::Error::new_spanned(
                struct_name,
                "Missing field marked with #[children]\n\
        Example:\n\
        #[children]\n\
        children: Scene,",
            ));
        }

        if events_field.is_none() {
            errors.push(syn::Error::new_spanned(
                struct_name,
                "Missing field marked with #[events]\n\
        Example:\n\
        #[events]\n\
        events: EventReceiver,",
            ));
        }

        let combined = errors
            .into_iter()
            .map(|e| e.to_compile_error())
            .collect::<proc_macro2::TokenStream>();

        return TokenStream::from(combined);
    }

    let transform = transform_field.unwrap();
    let children = children_field.unwrap();
    let events = events_field.unwrap();

    let expanded = quote! {

        impl ::maple::nodes::Node for #struct_name
            where
                #struct_name: Clone,
            {
            fn get_transform(&mut self) -> &mut ::maple::components::NodeTransform {
                &mut self.#transform
            }

            fn get_children(&self) -> &::maple::context::Scene {
                &self.#children
            }

            fn get_events(&mut self) -> &mut ::maple::components::EventReceiver {
                &mut self.#events
            }

            fn get_children_mut(&mut self) -> &mut ::maple::context::Scene {
                &mut self.#children
            }
        }
    };

    TokenStream::from(expanded)
}
