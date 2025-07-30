//! Helper crate providing procedural macros for the Rust engine of hax.
//!
//! Currently it provides the following.
//!  - Macros for deriving groups of traits.
//!    Most of the type from the AST have the same bounds, so that helps deduplicating a lot.
//!    Also, the fact those derive groups are named is helpful: for instance for code generation
//!    a simple `use derive_group_for_ast_base as derive_group_for_ast` can change what is to be
//!    derived without any attribute manipulation.

use proc_macro::TokenStream;
use proc_macro2::{Group, Ident, Span};
use quote::{ToTokens, quote};
use syn::{
    Field, FieldsUnnamed, Token, parse_macro_input, parse_quote, punctuated::Punctuated,
    token::Paren,
};
use utils::*;

mod utils {
    use super::*;
    pub(crate) fn crate_name() -> Ident {
        let krate = module_path!().split("::").next().unwrap();
        Ident::new(krate, Span::call_site())
    }

    /// Prepends a `proc_macro2::TokenStream` to a `TokenStream`
    pub(crate) fn prepend(item: TokenStream, prefix: proc_macro2::TokenStream) -> TokenStream {
        let item: proc_macro2::TokenStream = item.into();
        quote! {
            #prefix
            #item
        }
        .into()
    }

    /// Add a derive attribute to `item`
    pub(crate) fn add_derive(item: TokenStream, payload: proc_macro2::TokenStream) -> TokenStream {
        prepend(item, quote! {#[derive(#payload)]})
    }
}

/// Derive the common derives for the hax engine AST.
/// This is a equivalent to `derive_group_for_ast_serialization` and `derive_group_for_ast_base`.
#[proc_macro_attribute]
pub fn derive_group_for_ast(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let krate = crate_name();
    prepend(
        item,
        quote! {
            #[#krate::derive_group_for_ast_base]
            #[#krate::derive_group_for_ast_serialization]
        },
    )
}

/// Derive the necessary [de]serialization related traits for nodes in the AST.
#[proc_macro_attribute]
pub fn derive_group_for_ast_serialization(_attr: TokenStream, item: TokenStream) -> TokenStream {
    add_derive(
        item,
        quote! {::serde::Deserialize, ::serde::Serialize, ::schemars::JsonSchema},
    )
}

/// Derive the basic necessary traits for nodes in the AST.
#[proc_macro_attribute]
pub fn derive_group_for_ast_base(_attr: TokenStream, item: TokenStream) -> TokenStream {
    add_derive(
        item,
        quote! {Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord, derive_generic_visitor::Drive, derive_generic_visitor::DriveMut},
    )
}

#[proc_macro_attribute]
pub fn setup_derive_handling(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item: syn::ItemStruct = parse_macro_input!(item);
    if let fields @ syn::Fields::Unit = &mut item.fields {
        let span = Group::new(proc_macro2::Delimiter::Brace, fields.to_token_stream()).delim_span();
        *fields = syn::Fields::Unnamed(FieldsUnnamed {
            paren_token: Paren { span },
            unnamed: Punctuated::default(),
        })
    }
    fn fresh_ident(base: &str, existing: &[Ident]) -> Ident {
        let existing: std::collections::HashSet<_> =
            existing.iter().map(|id| id.to_string()).collect();

        (0..)
            .map(|i| {
                if i == 0 {
                    base.to_string()
                } else {
                    format!("{}{}", base, i)
                }
            })
            .find(|name| !existing.contains(name))
            .map(|name| Ident::new(&name, Span::call_site()))
            .expect("should always find a fresh identifier")
    }
    let (fields, named) = match &mut item.fields {
        syn::Fields::Named(fields_named) => (&mut fields_named.named, true),
        syn::Fields::Unnamed(fields_unnamed) => (&mut fields_unnamed.unnamed, false),
        syn::Fields::Unit => unreachable!(),
    };

    let existing_names = fields
        .iter()
        .flat_map(|f| &f.ident)
        .cloned()
        .collect::<Vec<_>>();

    let (extra_field_ident, extra_field_ident_ts) = if named {
        let ident = fresh_ident("error_handling_state", &existing_names);
        (Some(ident.clone()), ident.to_token_stream())
    } else {
        (
            None,
            syn::LitInt::new(&format!("{}", fields.len()), Span::call_site()).to_token_stream(),
        )
    };

    fields.push(Field {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        mutability: syn::FieldMutability::None,
        ident: extra_field_ident,
        colon_token: named.then_some(Token![:](Span::call_site())),
        ty: parse_quote! {crate::ast::visitors::wrappers::ErrorHandlingState},
    });

    let struct_name = &item.ident;
    let generics = &item.generics;
    quote! {
        #item
        impl #generics crate::ast::HasSpan for #struct_name #generics {
            fn span(&self) -> crate::ast::span::Span {
                self.#extra_field_ident_ts.0.clone()
            }
            fn span_mut(&mut self) -> &mut crate::ast::span::Span {
                &mut self.#extra_field_ident_ts.0
            }
        }
        impl #generics crate::ast::visitors::wrappers::VisitorWithErrors for #struct_name #generics {
            fn error_vault(&mut self) -> &mut crate::ast::visitors::wrappers::ErrorVault {
                &mut self.#extra_field_ident_ts.1
            }
        }
    }
    .into()
}
