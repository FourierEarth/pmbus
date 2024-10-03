use heck::ToSnakeCase;
use syn::{parse_quote, Ident, ItemFn};

use super::table::{CommandByteCount, CommandEntry, CommandIdent, CommandWrite};

pub struct WriteCommandFn(pub ItemFn);

impl WriteCommandFn {
    pub fn from_table_entry(entry: &CommandEntry) -> Option<Self> {
        let CommandEntry {
            span: _,
            byte: _,
            ident,
            write_kind,
            read_kind: _,
            byte_count,
        } = entry;

        let create_fn_ident = || match (ident, write_kind) {
            (CommandIdent::Verbatim(const_ident), CommandWrite::Write(_, _, _)) => {
                let base_ident = const_ident.to_string().to_snake_case();
                Ident::new(&format!("write_{base_ident}"), const_ident.span())
            }
            (CommandIdent::Verbatim(const_ident), CommandWrite::Send(_)) => {
                let base_ident = const_ident.to_string().to_snake_case();
                Ident::new(&format!("send_{base_ident}"), const_ident.span())
            }
            _ => unreachable!(),
        };

        match (ident, write_kind, byte_count) {
            (
                CommandIdent::Verbatim(command),
                CommandWrite::Write(_, _, ty),
                CommandByteCount::Count(_, 1),
            ) => {
                let write_fn_ident = create_fn_ident();
                Some(Self(parse_quote! {
                    async fn #write_fn_ident(&mut self, address: A, data: #ty) -> ::std::result::Result<(), <Self as ::embedded_hal::i2c::ErrorType>::Error> {
                        <Self as SmBus<A>>::write_byte(self, address, #command, data.into()).await
                    }
                }))
            }
            (
                CommandIdent::Verbatim(command),
                CommandWrite::Write(_, _, ty),
                CommandByteCount::Count(_, 2),
            ) => {
                let write_fn_ident = create_fn_ident();
                Some(Self(parse_quote! {
                    async fn #write_fn_ident(&mut self, address: A, data: #ty) -> ::std::result::Result<(), <Self as ::embedded_hal::i2c::ErrorType>::Error> {
                        <Self as SmBus<A>>::write_word(self, address, #command, data.into()).await
                    }
                }))
            }
            (
                CommandIdent::Verbatim(command),
                CommandWrite::Write(_, _, ty),
                CommandByteCount::Count(_, _),
            )
            | (
                CommandIdent::Verbatim(command),
                CommandWrite::Write(_, _, ty),
                CommandByteCount::Undefined(_),
            ) => {
                let write_fn_ident = create_fn_ident();
                Some(Self(parse_quote! {
                    async fn #write_fn_ident(&mut self, address: A, data: #ty) -> ::std::result::Result<(), <Self as ::embedded_hal::i2c::ErrorType>::Error> {
                        <Self as SmBus<A>>::block_write(self, address, #command, data.into()).await
                    }
                }))
            }

            // TODO: Make invalid definitions return errors.
            // (CommandIdent::Verbatim(_), CommandWrite::Send, _) => {
            //     todo!("error, send command can't have a byte count")
            // }
            _ => None,
        }
    }
}
