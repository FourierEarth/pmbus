use heck::ToSnakeCase;
use quote::{format_ident, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_quote, parse_quote_spanned, Ident, ItemFn, ItemTrait, Type};

use super::table::{
    CommandByteCount, CommandEntry, CommandIdent, CommandRead, CommandWrite, CommandsTable,
};

pub struct PmBusTraitItem(pub ItemTrait);

impl From<CommandsTable> for PmBusTraitItem {
    fn from(table: CommandsTable) -> Self {
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

        Self(parse_quote! {
            #[::async_trait::async_trait(?Send)]
            pub trait PmBus<A: ::embedded_hal::i2c::AddressMode = ::embedded_hal::i2c::SevenBitAddress>: SmBus<A> {
                #(#write_command_fns)*
                #(#read_command_fns)*
            }
        })
    }
}

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

pub struct ReadCommandFn(pub ItemFn);

impl ReadCommandFn {
    pub fn from_table_entry(entry: &CommandEntry) -> Option<Self> {
        let gen_read_fn = |read_op: Ident, command: &Ident, ty: &Type| -> ItemFn {
            let read_fn_ident = format_ident!(
                "read_{base_ident}",
                base_ident = Ident::new(&command.to_string().to_snake_case(), command.span())
            );
            parse_quote_spanned! {
                entry.span() =>
                async fn #read_fn_ident(&mut self, address: A) -> ::std::result::Result<#ty, <Self as ::embedded_hal::i2c::ErrorType>::Error> {
                    <Self as SmBus<A>>::#read_op(self, address, #command).await.map(Into::into)
                }
            }
        };

        // Currently all process calls are treated as "Block Read - Block Write Process Call" operations.
        // We will need to change this (or expand on it) as the write and return types become well-known.
        // Currently type is ignored.
        let gen_proc_call_fn = |command: &Ident, _ty: &Type| -> ItemFn {
            let call_fn_ident = format_ident!(
                "call_{base_ident}",
                base_ident = Ident::new(&command.to_string().to_snake_case(), command.span())
            );
            // TODO: The return value is fixed as a byte-vector.
            // Might be best to interpret the write type from another keyword in the write column.
            parse_quote_spanned! {
                entry.span() =>
                async fn #call_fn_ident(&mut self, address: A, write_block: &[u8]) -> ::std::result::Result<Vec<u8>, <Self as ::embedded_hal::i2c::ErrorType>::Error> {
                    <Self as SmBus<A>>::block_process_call(self, address, #command, write_block).await
                }
            }
        };

        match entry {
            // Discard entries which do not have a read operation.
            CommandEntry {
                read_kind: CommandRead::Undefined(_),
                ..
            }
            | CommandEntry {
                read_kind: CommandRead::Unimplemented(_),
                ..
            } => None,
            // Data length is one byte, the operation is `read_byte`.
            CommandEntry {
                ident: CommandIdent::Verbatim(command),
                read_kind: CommandRead::Read(_, _, ty),
                byte_count: CommandByteCount::Count(byte_count_span, 1),
                ..
            } => Some(Self(gen_read_fn(
                Ident::new("read_byte", *byte_count_span),
                command,
                ty,
            ))),
            // Data length is one byte, the operation is `read_word`.
            CommandEntry {
                ident: CommandIdent::Verbatim(command),
                read_kind: CommandRead::Read(_, _, ty),
                byte_count: CommandByteCount::Count(byte_count_span, 2),
                ..
            } => Some(Self(gen_read_fn(
                Ident::new("read_word", *byte_count_span),
                command,
                ty,
            ))),
            // The data size is known, but it is not a byte or a word.
            // The operation is `block_write`.
            CommandEntry {
                ident: CommandIdent::Verbatim(command),
                read_kind: CommandRead::Read(_, _, ty),
                byte_count: CommandByteCount::Count(byte_count_span, _),
                ..
            } => Some(Self(gen_read_fn(
                Ident::new("block_read", *byte_count_span),
                command,
                ty,
            ))),
            // Write kind is known, but data length undefined. This means we treat the data as a variable-sized block.
            // The operation is `block_write`.
            CommandEntry {
                ident: CommandIdent::Verbatim(command),
                read_kind: CommandRead::Read(_, _, ty),
                byte_count: CommandByteCount::Undefined(underscore),
                ..
            } => Some(Self(gen_read_fn(
                Ident::new("block_read", underscore.span()),
                command,
                ty,
            ))),
            // Process calls have many variations that need to be accounted for.
            // For example, the write data could be two bytes and the data read back is variable, or a fixed size.
            // The current `SmBus` trait just treats all process calls the same, as block-write and block-read.
            // More data can be added to the commands table to indicate the types in either direction.
            // Coercion does not work for these commands at the moment. Byte count is ignored, but refers to the size of the data read back.
            CommandEntry {
                ident: CommandIdent::Verbatim(command),
                read_kind: CommandRead::Call(_, _, ty),
                byte_count: CommandByteCount::Count(_byte_count_span, _),
                ..
            } => Some(Self(gen_proc_call_fn(command, ty))),
            // TODO: This is no different from the above, for now.
            // I expect this to be removed later as very few commands are actually variable sized.
            CommandEntry {
                ident: CommandIdent::Verbatim(command),
                read_kind: CommandRead::Call(_, _, ty),
                byte_count: CommandByteCount::Undefined(_),
                ..
            } => Some(Self(gen_proc_call_fn(command, ty))),
            // TODO: See comment TEST VALIDATION PATTERN (in `ReadCommandFn`).
            _ => {
                println!("{}", entry.to_token_stream());
                unimplemented!("edge cases and invalid entries not yet handled")
            }
        }
    }
}
