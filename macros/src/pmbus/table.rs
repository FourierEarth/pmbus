use proc_macro2::Span;
use quote::{quote, quote_spanned, ToTokens};
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
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

impl ToTokens for CommandIdent {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Undefined(underscore) => underscore.to_tokens(tokens),
            Self::Verbatim(ident) => ident.to_tokens(tokens),
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

impl ToTokens for CommandWrite {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Undefined(underscore) => underscore.to_tokens(tokens),
            Self::Unimplemented(never) => never.to_tokens(tokens),
            Self::Write(write, colon, ty) => quote!(#write #colon #ty).to_tokens(tokens),
            Self::Send(send) => send.to_tokens(tokens),
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

impl ToTokens for CommandRead {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            CommandRead::Undefined(underscore) => underscore.to_tokens(tokens),
            CommandRead::Unimplemented(never) => never.to_tokens(tokens),
            CommandRead::Read(read, colon, ty) => quote!(#read #colon #ty).to_tokens(tokens),
            CommandRead::Call(call, colon, ty) => quote!(#call #colon #ty).to_tokens(tokens),
        }
    }
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

impl ToTokens for CommandByteCount {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Undefined(underscore) => underscore.to_tokens(tokens),
            Self::Unimplemented(never) => never.to_tokens(tokens),
            Self::Count(span, byte_count) => quote_spanned!(*span => #byte_count).to_tokens(tokens),
        }
    }
}

pub struct CommandEntry {
    pub span: Span,
    pub byte: (Span, u8),
    pub ident: CommandIdent,
    pub write_kind: CommandWrite,
    pub read_kind: CommandRead,
    pub byte_count: CommandByteCount,
}

impl Parse for CommandEntry {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // // Record the position at the start of the first `|`.
        // let span = input.span();
        let left_pipe = input.parse::<Token![|]>()?;

        let byte = {
            let lit = input.parse::<LitInt>()?;
            (lit.span(), lit.base10_parse()?)
        };
        let ident = input.parse::<Token![|]>().and_then(|_| input.parse())?;
        let write_kind = input.parse::<Token![|]>().and_then(|_| input.parse())?;
        let read_kind = input.parse::<Token![|]>().and_then(|_| input.parse())?;
        let byte_count = input.parse::<Token![|]>().and_then(|_| input.parse())?;

        // TODO: Works on nightly only.
        // // The start and end tokens will never be from different files.
        // let span = span.join(input.parse::<Token![|]>()?.span()).unwrap();
        let right_pipe = input.parse::<Token![|]>()?;

        // Lossy.
        let span = quote!(#left_pipe #right_pipe).span();

        Ok(Self {
            span,
            byte,
            ident,
            write_kind,
            read_kind,
            byte_count,
        })
    }
}

impl ToTokens for CommandEntry {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            span,
            byte,
            ident,
            write_kind,
            read_kind,
            byte_count,
        } = self;
        let byte = byte.1;

        quote_spanned! {
            *span =>
            | #byte | #ident | #write_kind | #read_kind | #byte_count |
        }
        .to_tokens(tokens)
    }
}

pub struct CommandsTable(pub Punctuated<CommandEntry, Token![,]>);

impl Parse for CommandsTable {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Punctuated::parse_terminated(input).map(Self)
    }
}

impl ToTokens for CommandsTable {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens)
    }
}
