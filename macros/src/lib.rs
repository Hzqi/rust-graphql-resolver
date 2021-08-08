use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive DataValue for struct
/// ```compile_fail
/// #[derive(Debug, Clone, GraphQLDataValue)]
/// struct HelloWorld {
///     hello: String,
///     greeting: String,
/// }
/// ```
#[proc_macro_derive(GraphQLDataValue)]
pub fn graphql_data_value(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;

    if let syn::Data::Struct(struct_data) = ast.data {
        let field_quotes = struct_data
            .fields
            .iter()
            .map(|ref field| -> TokenStream2 {
                let field_ident = field.ident.clone().unwrap();
                let field_name = field_ident.to_string();
                quote! {
                    (#field_name.to_string(), self.#field_ident.to_data_value()),
                }
            })
            .reduce(|mut a, b| -> TokenStream2 {
                a.extend(b);
                a
            })
            .unwrap();

        let quoted_code = quote! {
            impl rust_graphql_resolver::value::ToDataValue for #struct_name {
                fn to_data_value(&self) -> rust_graphql_resolver::value::DataValue {
                    use std::{array::IntoIter, collections::BTreeMap, iter::FromIterator};

                    rust_graphql_resolver::value::DataValue::Object(BTreeMap::from_iter(IntoIter::new([
                        #field_quotes
                    ])))
                }
            }
        };
        proc_macro::TokenStream::from(quoted_code)
    } else {
        // not a struct
        panic!("#[derive(GraphQLDataValue)] is only defined for structs");
    }
}
