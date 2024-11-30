extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parser, parse_macro_input, Data, DataStruct, DeriveInput, Fields};

#[proc_macro_attribute]
pub fn add_node_fields(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as syn::ItemStruct);

    // Example: Add fields `health` and `points` to the struct
    let new_fields: syn::FieldsNamed = syn::parse_quote!({
        transform: NodeTransform,
        ready_callback: Option<Box<dyn FnMut(&mut Self)>>,
        behavior_callback: Option<Box<dyn FnMut(&mut Self, &mut GameContext)>>,
    });

    if let syn::Fields::Named(ref mut fields) = input.fields {
        fields.named.extend(new_fields.named);
    }

    // Return the modified struct
    TokenStream::from(quote! {
        #input
    })
}

#[proc_macro_derive(Node)]
pub fn derive_node(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Get the fields from the struct
    let fields = if let Data::Struct(DataStruct {
        fields: Fields::Named(ref fields),
        ..
    }) = input.data
    {
        fields
    } else {
        return TokenStream::from(quote! {
            compile_error!("`Node` can only be derived for structs with named fields.");
        });
    };

    // Extract the user-defined field names and generate default initializations
    let user_field_initializers = fields.named.iter().map(|field| {
        let field_name = &field.ident;
        // If a field type implements Default, use `Default::default()`; otherwise use placeholder values
        quote! {
            #field_name: Default::default()
        }
    });

    // Generate the `new` method which includes both user-defined and injected fields
    let expanded = quote! {
            impl #struct_name {
                pub fn new() -> Self {
                    Self {
                        transform: NodeTransform::default(),
                        #(#user_field_initializers),*
                    }
                }
            }

            impl Node for #struct_name {
            type Transform = NodeTransform;

            // Return by value to avoid type mismatch
            fn get_model_matrix(&self) -> glm::Mat4 {
                self.transform.matrix.clone()
            }

            fn get_transform(&self) -> &Self::Transform {
                &self.transform
            }
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn define_node(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input struct
    let mut input = parse_macro_input!(item as syn::ItemStruct);
    let struct_name = &input.ident;

    // Ensure the struct has named fields
    let user_fields = if let syn::Fields::Named(ref fields) = input.fields {
        fields.clone()
    } else {
        return TokenStream::from(quote! {
            compile_error!("`define_node` can only be used on structs with named fields.");
        });
    };

    // Determine if the struct already has any of the additional fields
    let mut has_transform = false;

    for field in &user_fields.named {
        if let Some(field_name) = &field.ident {
            match field_name.to_string().as_str() {
                "transform" => has_transform = true,
                _ => {}
            }
        }
    }

    // Add missing fields
    let mut additional_fields: syn::FieldsNamed = syn::parse_quote!({});
    if !has_transform {
        additional_fields.named.push(
            syn::Field::parse_named
                .parse2(quote! {
                    transform: NodeTransform
                })
                .unwrap(),
        );
    }

    // Add additional fields to the struct
    if let syn::Fields::Named(ref mut fields) = input.fields {
        fields.named.extend(additional_fields.named);
    }

    // Generate field initializers for user-defined fields
    let user_field_initializers = user_fields.named.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            #field_name: Default::default()
        }
    });

    // Generate field initializers for additional fields
    let additional_initializers = vec![if !has_transform {
        quote! { transform: NodeTransform::default() }
    } else {
        quote! {}
    }]
    .into_iter()
    .filter(|token| !token.is_empty());

    // Generate the `impl` block for the Node trait
    let expanded = quote! {
        #input

        impl #struct_name {
            pub fn new() -> Self {
                Self {
                    #(#user_field_initializers),*,
                    #(#additional_initializers),*
                }
            }
        }

        impl Node for #struct_name {
            type Transform = NodeTransform;

            fn get_model_matrix(&self) -> glm::Mat4 {
                self.transform.matrix.clone()
            }

            fn get_transform(&self) -> &Self::Transform {
                &self.transform
            }
        }
    };

    TokenStream::from(expanded)
}
