use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct, LitStr};

#[proc_macro_attribute]
pub fn ce_commend(attr: TokenStream, item: TokenStream) -> TokenStream {
    let command_str = parse_macro_input!(attr as LitStr);
    let command_value = command_str.value();
    let input_struct = parse_macro_input!(item as ItemStruct);
    let ident = &input_struct.ident;
    let expanded = quote! {
        #input_struct
        impl CECommandMarker for #ident {
            const COMMAND: &'static str = #command_value;
        }
        inventory::submit! {
            ClientEventRegistry {
                command: <#ident as CECommandMarker>::COMMAND,
                parse_fn: <#ident as ClientEvent>::parse,
            }
        }
    };
    expanded.into()
}
