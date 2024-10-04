mod pmbus;

use proc_macro::TokenStream as TokenStream1;
use quote::quote;
use syn::parse_macro_input;

use self::pmbus::constants::CommandConstants;
use self::pmbus::table::CommandsTable;
use self::pmbus::trait_impl::PmBusTraitItem;

#[proc_macro]
pub fn impl_commands(input: TokenStream1) -> TokenStream1 {
    let table: CommandsTable = parse_macro_input!(input);
    let constants = CommandConstants::from(&table).0;
    let pmbus_trait = PmBusTraitItem::from(table).0;
    quote! {
        #(#constants)*
        #pmbus_trait
    }
    .into()
}
