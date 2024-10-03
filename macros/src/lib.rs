mod pmbus;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::parse_macro_input;

use self::pmbus::table::{CommandIdent, CommandsTable};

#[proc_macro]
pub fn impl_commands(input: TokenStream1) -> TokenStream1 {
    let table: CommandsTable = parse_macro_input!(input);

    let mut constants = TokenStream2::new();

    for entry in table.0.iter() {
        match &entry.ident {
            CommandIdent::Undefined => continue,
            CommandIdent::Verbatim(ident) => {
                // Maybe it would have been better to just keep the `LitInt` around, because now we have to format it as hexadecimal again.
                let byte = entry.byte.1;
                quote! {
                    pub const #ident: u8 = #byte;
                }
                .to_tokens(&mut constants);
            }
        }
    }

    constants.into()
}
