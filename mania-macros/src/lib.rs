use md5::{Digest, Md5};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, ItemFn, ItemStruct, LitStr, Path, parse_macro_input, punctuated::Punctuated,
};

#[proc_macro_attribute]
pub fn commend(attr: TokenStream, item: TokenStream) -> TokenStream {
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

#[proc_macro_derive(ServerEvent)]
pub fn derive_server_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;
    let expanded = quote! {
        impl ServerEvent for #struct_name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn handle_event(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let parser = Punctuated::<Path, syn::Token![,]>::parse_terminated;
    let event_paths = parse_macro_input!(attr with parser);

    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_block = &input_fn.block;
    let fn_sig = &input_fn.sig;
    let user_fn = quote! {
        #fn_vis #fn_sig {
            #fn_block
        }
    };

    let mut registrations = proc_macro2::TokenStream::new();

    for event_path in event_paths {
        let mut hasher = Md5::new();
        let fn_token_stream = user_fn.to_string();
        let event_path_str = quote! { #event_path }.to_string();
        hasher.update(&fn_token_stream);
        hasher.update(&event_path_str);
        let hash_value = hex::encode(hasher.finalize());

        let type_id_fn_name =
            syn::Ident::new(&format!("_mhe_type_id_{}", hash_value), fn_name.span());
        let wrapper_fn_name =
            syn::Ident::new(&format!("_mhe_wrap_id_{}", hash_value), fn_name.span());

        let trait_check = quote! {
            const _: () = {
                struct Checker<T: crate::core::event::ServerEvent>(core::marker::PhantomData<T>);
                let _ = Checker::<#event_path>(core::marker::PhantomData);
            };
        };

        let code_for_one_event = quote! {
            #trait_check

            fn #type_id_fn_name() -> ::std::any::TypeId {
                ::std::any::TypeId::of::<#event_path>()
            }

            fn #wrapper_fn_name<'a>(
                event: &'a mut dyn crate::core::event::ServerEvent,
                handle: std::sync::Arc<crate::core::business::BusinessHandle>,
                flow: crate::core::business::LogicFlow,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = &'a dyn ServerEvent> + Send + 'a>> {
                Box::pin(#fn_name(event, handle, flow))
            }

            inventory::submit! {
                LogicRegistry {
                    event_type_id_fn: #type_id_fn_name,
                    event_handle_fn: #wrapper_fn_name,
                }
            }
        };
        registrations.extend(code_for_one_event);
    }

    let expanded = quote! {
        #user_fn
        #registrations
    };
    expanded.into()
}
