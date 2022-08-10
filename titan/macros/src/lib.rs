use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, ExprCall, ItemStruct, PathArguments, Type};

#[proc_macro_attribute]
pub fn component(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct =
        parse::<ItemStruct>(TokenStream::from(input)).expect("Could not parse item struct");
    let item_struct_name = item_struct.ident.clone();
    let item_struct_name_string = item_struct_name.to_string();
    let expanded = quote! {
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        #item_struct
        impl titan::ComponentMeta for #item_struct_name {
            fn get_component_kind() -> titan::ComponentKind {
                titan::ComponentKind(String::from(#item_struct_name_string))
            }
        }
        impl titan::ComponentMeta for &#item_struct_name {
            fn get_component_kind() -> titan::ComponentKind {
                titan::ComponentKind(String::from(#item_struct_name_string))
            }
        }
        impl titan::ComponentMeta for &mut #item_struct_name {
            fn get_component_kind() -> titan::ComponentKind {
                titan::ComponentKind(String::from(#item_struct_name_string))
            }
        }
    };
    TokenStream::from(expanded)
}
