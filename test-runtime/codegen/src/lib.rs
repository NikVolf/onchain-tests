
//! Provides macros for async runtime of Gear programs.

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;

#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = syn::parse_macro_input!(item as syn::ItemFn);
    let ident = &function.sig.ident;
    let extern_ident = Ident::new(&format!("test_{}", ident), Span::call_site());

    quote! {
        #function

        #[no_mangle]
        pub unsafe extern "C" fn #extern_ident() {
            let test_future = gear_test_runtime::ContextFuture::new(
                async {
                    let session = gear_test_runtime::active_session();
                    #ident(&session).await;
                },
                concat!(module_path!(), "::", stringify!(#ident)),
            );

            gear_test_runtime::CONTEXT_FUTURES.push(test_future);
        }
    }
    .into()
}
