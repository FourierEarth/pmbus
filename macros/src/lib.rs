use proc_macro::TokenStream as TokenStream1;
use quote::quote;

// use proc_macro2::TokenStream as TokenStream2;

#[proc_macro]
pub fn impl_commands(input: TokenStream1) -> TokenStream1 {
    quote! {
        #[derive(Copy, Clone, PartialEq, Eq, Debug)]
        pub enum Command {
        }
    }
    .into()
}
