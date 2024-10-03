use proc_macro::TokenStream as TokenStream1;

// use proc_macro2::TokenStream as TokenStream2;

#[proc_macro]
pub fn impl_commands(input: TokenStream1) -> TokenStream1 {
    input
}
