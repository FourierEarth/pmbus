mod pmbus;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::parse_macro_input;

use self::pmbus::table::{CommandIdent, CommandsTable};
use self::pmbus::trait_impl::WriteCommandFn;

#[proc_macro]
pub fn impl_commands(input: TokenStream1) -> TokenStream1 {
    let table: CommandsTable = parse_macro_input!(input);

    let mut const_tokens = TokenStream2::new();
    for entry in table.0.iter() {
        match &entry.ident {
            CommandIdent::Undefined(_) => continue,
            CommandIdent::Verbatim(ident) => {
                // Maybe it would have been better to just keep the `LitInt` around, because now we have to format it as hexadecimal again.
                let byte = entry.byte.1;
                quote! {
                    pub const #ident: u8 = #byte;
                }
                .to_tokens(&mut const_tokens);
            }
        }
    }

    let write_command_fns = table
        .0
        .iter()
        .filter_map(|entry| WriteCommandFn::from_table_entry(entry).map(|write| write.0));

    let trait_tokens = quote! {
        pub trait PmBus<A: ::embedded_hal::i2c::AddressMode = ::embedded_hal::i2c::SevenBitAddress>: SmBus<A> {
            #(#write_command_fns)*
        }
    };

    let mut output = TokenStream2::new();

    output.extend(const_tokens);
    output.extend(trait_tokens);

    output.into()
}
