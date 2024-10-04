mod pmbus;

use proc_macro::TokenStream as TokenStream1;
use quote::quote;
use syn::parse_macro_input;

use self::pmbus::constants::CommandConstants;
use self::pmbus::table::CommandsTable;
use self::pmbus::trait_impl::{ReadCommandFn, WriteCommandFn};

#[proc_macro]
pub fn impl_commands(input: TokenStream1) -> TokenStream1 {
    let table: CommandsTable = parse_macro_input!(input);

    let constants = CommandConstants::from(&table).0;

    // TODO: Stop mapping to the inner value. I'm leaving this alone for now because
    // I expect it to change significantly once the structure of read and write data is better defined.
    let write_command_fns = table
        .0
        .iter()
        .filter_map(|entry| WriteCommandFn::from_table_entry(entry).map(|write| write.0));
    let read_command_fns = table
        .0
        .iter()
        .filter_map(|entry| ReadCommandFn::from_table_entry(entry).map(|write| write.0));

    quote! {
        #(#constants)*

        #[::async_trait::async_trait(?Send)]
        pub trait PmBus<A: ::embedded_hal::i2c::AddressMode = ::embedded_hal::i2c::SevenBitAddress>: SmBus<A> {
            #(#write_command_fns)*
            #(#read_command_fns)*
        }
    }.into()
}
