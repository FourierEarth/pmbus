use proc_macro::TokenStream as TokenStream1;
use quote::quote;
use syn::parse_macro_input;

use self::pmbus::CommandsTable;

#[proc_macro]
pub fn impl_commands(input: TokenStream1) -> TokenStream1 {
    let table: CommandsTable = parse_macro_input!(input);
    dbg!(table);
    quote! {
        #[derive(Copy, Clone, PartialEq, Eq, Debug)]
        pub enum Command {
        }
    }
    .into()
}

mod pmbus {
    use std::fmt::Debug;

    use proc_macro2::Span;
    use quote::ToTokens;
    use syn::parse::Parse;
    use syn::punctuated::Punctuated;
    use syn::{Ident, LitInt, Token, Type};

    mod kw {
        syn::custom_keyword!(write);
        syn::custom_keyword!(send);
        syn::custom_keyword!(read);
        syn::custom_keyword!(call);
    }

    // TODO: Preserve tokens/spans?
    #[derive(Debug)]
    pub enum CommandIdent {
        // UNDEFINED / RESERVED∏
        Undefined,
        // CONSTANT IDENTIFIER
        Verbatim(String),
    }

    impl Parse for CommandIdent {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            // TODO: Started using lookahead for error handling, can't decide which way to go.
            // Can use `ParseStream::peak` instead of `Lookahead::peek` instead as we aren't getting
            // any benefits form the latter (without using its generated error).
            let look = input.lookahead1();
            if look.peek(Token![_]) {
                input.parse::<Token![_]>().unwrap();
                Ok(Self::Undefined)
            } else {
                input
                    .parse::<Ident>()
                    .map(|ident| Self::Verbatim(ident.to_string()))
            }
        }
    }

    // TODO: Preserve tokens/spans?
    pub enum CommandWrite {
        // UNDEFINED / RESERVED
        Undefined,
        // UNIMPLEMENTED / MANUFACTURER
        Unimplemented,
        // WRITE TYPE
        Write(Type),
        // SEND COMMAND BIT
        Send,
    }

    impl Parse for CommandWrite {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let look = input.lookahead1();
            if look.peek(Token![_]) {
                // UNDEFINED / RESERVED
                input.parse::<Token![_]>().unwrap();
                Ok(Self::Undefined)
            } else if look.peek(Token![!]) {
                // UNIMPLEMENTED / MANUFACTURER
                input.parse::<Token![!]>().unwrap();
                Ok(Self::Unimplemented)
            } else if look.peek(kw::write) {
                // WRITE TYPE
                input.parse::<kw::write>().unwrap();
                input.parse::<Token![:]>()?;
                input.parse::<Type>().map(Self::Write)
            } else if look.peek(kw::send) {
                // SEND COMMAND BIT
                input.parse::<kw::send>().unwrap();
                Ok(Self::Send)
            } else {
                Err(look.error())
            }
        }
    }

    impl Debug for CommandWrite {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                CommandWrite::Undefined => write!(f, "UNDEFINED"),
                CommandWrite::Unimplemented => write!(f, "UNIMPLEMENTED"),
                CommandWrite::Write(ty) => write!(f, "{}", ty.to_token_stream()),
                CommandWrite::Send => write!(f, "SEND"),
            }
        }
    }

    // TODO: Preserve tokens/spans?
    pub enum CommandRead {
        // UNDEFINED / RESERVED
        Undefined,
        // UNIMPLEMENTED / MANUFACTURER
        Unimplemented,
        // READ TYPE
        Read(Type),
        // PROCEDURE CALL TYPE
        Call(Type),
    }

    impl Parse for CommandRead {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let look = input.lookahead1();
            if look.peek(Token![_]) {
                // UNDEFINED / RESERVED
                input.parse::<Token![_]>().unwrap();
                Ok(Self::Undefined)
            } else if look.peek(Token![!]) {
                // UNIMPLEMENTED / MANUFACTURER
                input.parse::<Token![!]>().unwrap();
                Ok(Self::Unimplemented)
            } else if look.peek(kw::read) {
                // READ TYPE
                input.parse::<kw::read>().unwrap();
                input.parse::<Token![:]>()?;
                input.parse::<Type>().map(Self::Read)
            } else if look.peek(kw::call) {
                // PROCEDURE CALL TYPE
                input.parse::<kw::call>().unwrap();
                input.parse::<Token![:]>()?;
                input.parse::<Type>().map(Self::Call)
            } else {
                Err(look.error())
            }
        }
    }

    impl Debug for CommandRead {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Undefined => write!(f, "UNDEFINED"),
                Self::Unimplemented => write!(f, "UNIMPLEMENTED"),
                Self::Read(ty) => write!(f, "{}", ty.into_token_stream()),
                Self::Call(ty) => write!(f, "{}", ty.into_token_stream()),
            }
        }
    }

    #[derive(Debug)]
    pub enum CommandByteCount {
        // UNDEFINED / RESERVED
        Undefined,
        // UNIMPLEMENTED / MANUFACTURER
        Unimplemented,
        // KNOWN QUANTITY
        Count(Span, u8),
    }

    impl Parse for CommandByteCount {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            // TODO: More worries about error handling after using lookahead fork.
            let look = input.lookahead1();
            if look.peek(Token![_]) {
                input.parse::<Token![_]>().unwrap();
                Ok(Self::Undefined)
            } else if look.peek(Token![!]) {
                input.parse::<Token![!]>().unwrap();
                Ok(Self::Unimplemented)
            } else {
                input
                    .parse::<LitInt>()
                    .and_then(|lit| Ok(Self::Count(lit.span(), lit.base10_parse()?)))
            }
        }
    }

    #[derive(Debug)]
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

    impl Debug for CommandsTable {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for entry in self.0.iter() {
                writeln!(f, "{:?}", entry)?;
            }
            Ok(())
        }
    }
}
