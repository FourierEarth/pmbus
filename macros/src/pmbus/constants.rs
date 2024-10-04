use quote::ToTokens;
use syn::{parse_quote, ItemConst};

use super::table::{CommandIdent, CommandsTable};

pub struct CommandConstants(pub Vec<ItemConst>);

impl From<&CommandsTable> for CommandConstants {
    fn from(table: &CommandsTable) -> Self {
        Self(
            table
                .0
                .iter()
                .filter_map(|entry| match &entry.ident {
                    CommandIdent::Undefined(_) => None,
                    CommandIdent::Verbatim(ident) => {
                        let byte = &entry.byte;
                        Some(parse_quote! {
                            pub const #ident: u8 = #byte;
                        })
                    }
                })
                .collect(),
        )
    }
}

impl ToTokens for CommandConstants {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for command in &self.0 {
            command.to_tokens(tokens);
        }
    }
}
