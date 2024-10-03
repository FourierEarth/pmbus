use proc_macro2::Span;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::{Ident, LitInt, Token, Type};

mod kw {
    syn::custom_keyword!(write);
    syn::custom_keyword!(send);
    syn::custom_keyword!(read);
    syn::custom_keyword!(call);
}

pub enum CommandIdent {
    // UNDEFINED / RESERVED
    Undefined(Token![_]),
    // CONSTANT IDENTIFIER
    Verbatim(Ident),
}

impl Parse for CommandIdent {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![_]) {
            input.parse().map(Self::Undefined)
        } else {
            input.parse().map(Self::Verbatim)
        }
    }
}

pub enum CommandWrite {
    // UNDEFINED / RESERVED
    Undefined(Token![_]),
    // UNIMPLEMENTED / MANUFACTURER
    Unimplemented(Token![!]),
    // WRITE TYPE
    Write(kw::write, Token![:], Type),
    // SEND COMMAND BIT
    Send(kw::send),
}

impl Parse for CommandWrite {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let look = input.lookahead1();
        if look.peek(Token![_]) {
            // UNDEFINED / RESERVED
            input.parse().map(Self::Undefined)
        } else if look.peek(Token![!]) {
            // UNIMPLEMENTED / MANUFACTURER
            input.parse().map(Self::Unimplemented)
        } else if look.peek(kw::write) {
            // WRITE TYPE
            Ok(Self::Write(input.parse()?, input.parse()?, input.parse()?))
        } else if look.peek(kw::send) {
            // SEND COMMAND BIT
            input.parse().map(Self::Send)
        } else {
            Err(look.error())
        }
    }
}

pub enum CommandRead {
    // UNDEFINED / RESERVED
    Undefined(Token![_]),
    // UNIMPLEMENTED / MANUFACTURER
    Unimplemented(Token![!]),
    // READ TYPE
    Read(kw::read, Token![:], Type),
    // PROCEDURE CALL TYPE
    Call(kw::call, Token![:], Type),
}

impl Parse for CommandRead {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let look = input.lookahead1();
        if look.peek(Token![_]) {
            // UNDEFINED / RESERVED
            input.parse().map(Self::Undefined)
        } else if look.peek(Token![!]) {
            // UNIMPLEMENTED / MANUFACTURER
            input.parse().map(Self::Unimplemented)
        } else if look.peek(kw::read) {
            // READ TYPE
            Ok(Self::Read(input.parse()?, input.parse()?, input.parse()?))
        } else if look.peek(kw::call) {
            // PROCEDURE CALL TYPE
            Ok(Self::Call(input.parse()?, input.parse()?, input.parse()?))
        } else {
            Err(look.error())
        }
    }
}

pub enum CommandByteCount {
    // UNDEFINED / RESERVED
    Undefined(Token![_]),
    // UNIMPLEMENTED / MANUFACTURER
    Unimplemented(Token![!]),
    // KNOWN QUANTITY
    Count(Span, u8),
}

impl Parse for CommandByteCount {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // TODO: Not taking advantage of `Lookahead1`'s error helper.
        let look = input.lookahead1();
        if look.peek(Token![_]) {
            input.parse().map(CommandByteCount::Undefined)
        } else if look.peek(Token![!]) {
            input.parse().map(CommandByteCount::Unimplemented)
        } else {
            input
                .parse::<LitInt>()
                .and_then(|lit| Ok(Self::Count(lit.span(), lit.base10_parse()?)))
        }
    }
}

pub struct CommandEntry {
    pub byte: (Span, u8),
    pub ident: CommandIdent,
    pub write_kind: CommandWrite,
    pub read_kind: CommandRead,
    pub byte_count: CommandByteCount,
}

impl Parse for CommandEntry {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let byte = input.parse::<Token![|]>().and_then(|_| {
            let lit = input.parse::<LitInt>()?;
            Ok((lit.span(), lit.base10_parse()?))
        })?;
        let ident = input.parse::<Token![|]>().and_then(|_| input.parse())?;
        let write_kind = input.parse::<Token![|]>().and_then(|_| input.parse())?;
        let read_kind = input.parse::<Token![|]>().and_then(|_| input.parse())?;
        let byte_count = input.parse::<Token![|]>().and_then(|_| input.parse())?;
        let _ = input.parse::<Token![|]>()?;
        Ok(Self {
            byte,
            ident,
            write_kind,
            read_kind,
            byte_count,
        })
    }
}
pub struct CommandsTable(pub Punctuated<CommandEntry, Token![,]>);

impl Parse for CommandsTable {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Punctuated::parse_terminated(input).map(Self)
    }
}
