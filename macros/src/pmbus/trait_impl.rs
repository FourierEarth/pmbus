use heck::ToSnakeCase;
use quote::{format_ident, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_quote_spanned, Ident, ItemFn, Type};

use super::table::{CommandByteCount, CommandEntry, CommandIdent, CommandWrite};

// TODO: This structure is mainly associated with logic,
// which should probably be moved elsewhere (such as to a new-type wrapper around `ItemTrait`).
pub struct WriteCommandFn(pub ItemFn);

impl WriteCommandFn {
    pub fn from_table_entry(entry: &CommandEntry) -> Option<Self> {
        let gen_send_fn = |command: &Ident| -> ItemFn {
            let write_fn_ident = format_ident!(
                "write_{base_ident}",
                base_ident = Ident::new(&command.to_string().to_snake_case(), command.span())
            );
            parse_quote_spanned! {
                entry.span() =>
                async fn #write_fn_ident(&mut self, address: A) -> ::std::result::Result<(), <Self as ::embedded_hal::i2c::ErrorType>::Error> {
                    <Self as SmBus<A>>::send_byte(self, address, #command).await
                }
            }
        };

        let gen_write_fn = |write_op: Ident, command: &Ident, ty: &Type| -> ItemFn {
            let send_fn_ident = format_ident!(
                "send_{base_ident}",
                base_ident = Ident::new(&command.to_string().to_snake_case(), command.span())
            );
            parse_quote_spanned! {
                entry.span() =>
                async fn #send_fn_ident(&mut self, address: A, data: #ty) -> ::std::result::Result<(), <Self as ::embedded_hal::i2c::ErrorType>::Error> {
                    <Self as SmBus<A>>::#write_op(self, address, #command, data).await
                }
            }
        };

        match entry {
            // TRIVIAL VALIDATION PATTERN
            // This needs to be somewhere else, this constructor for `WriteCommandFn`
            // will need at least one more refractor.
            //
            // TODO: I would like to see read-only, undefined, and unimplemented entries removed
            // before they ever get to this constructor. In this way, we can remove `Option` from the return type,
            // replacing it with `syn::Result` and add arms to validate *specifically* write command entries.
            //
            // If `!` is used to indicate that a command is unimplemented,
            // all fields after the command identifier are expected to be `!` also.
            // CommandEntry {
            //     ident: CommandIdent::Verbatim(_),
            //     write_kind: CommandWrite::Unimplemented(_),
            //     read_kind: CommandRead::Unimplemented(_),
            //     byte_count: CommandByteCount::Unimplemented(_),
            //     ..
            // } => None,
            // Discard entries which do not have a write operation.
            CommandEntry {
                write_kind: CommandWrite::Undefined(_),
                ..
            }
            | CommandEntry {
                write_kind: CommandWrite::Unimplemented(_),
                ..
            } => None,
            // Data length is one byte, the operation is `write_byte`.
            CommandEntry {
                ident: CommandIdent::Verbatim(command),
                write_kind: CommandWrite::Write(_, _, ty),
                byte_count: CommandByteCount::Count(byte_count_span, 1),
                ..
            } => Some(Self(gen_write_fn(
                Ident::new("write_byte", *byte_count_span),
                command,
                ty,
            ))),
            // Data length is two bytes, the operation is `write_word`.
            CommandEntry {
                ident: CommandIdent::Verbatim(command),
                write_kind: CommandWrite::Write(_, _, ty),
                byte_count: CommandByteCount::Count(byte_count_span, 2),
                ..
            } => Some(Self(gen_write_fn(
                Ident::new("write_word", *byte_count_span),
                command,
                ty,
            ))),
            // The data size is known, but it is not a byte or a word.
            // The operation is `block_write`.
            CommandEntry {
                ident: CommandIdent::Verbatim(command),
                write_kind: CommandWrite::Write(_, _, ty),
                byte_count: CommandByteCount::Count(byte_count_span, _),
                ..
            } => Some(Self(gen_write_fn(
                Ident::new("block_write", *byte_count_span),
                command,
                ty,
            ))),
            // Write kind is known, but data length undefined. This means we treat the data as a variable-sized block.
            // The operation is `block_write`.
            CommandEntry {
                ident: CommandIdent::Verbatim(command),
                write_kind: CommandWrite::Write(_, _, ty),
                byte_count: CommandByteCount::Undefined(underscore),
                ..
            } => Some(Self(gen_write_fn(
                Ident::new("block_write", underscore.span()),
                command,
                ty,
            ))),
            // The `write_kind` is `Send` and the `byte_count` is `0`, so the operation is `send_byte`.
            // Using `send` with nonzero data size is expressly prohibited.
            CommandEntry {
                ident: CommandIdent::Verbatim(command),
                write_kind: CommandWrite::Send(_),
                byte_count: CommandByteCount::Count(_, 0),
                ..
            } => Some(Self(gen_send_fn(command))),
            // TODO: See comment TEST VALIDATION PATTERN.
            _ => {
                println!("{}", entry.to_token_stream());
                unimplemented!("edge cases and invalid entries not yet handled")
            }
        }
    }
}
