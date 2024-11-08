use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, parse_macro_input, Expr, Token};

struct ParsedSyscallInputEntry(Expr);

struct ParsedSyscallInput {
    entries: Vec<ParsedSyscallInputEntry>,
}

impl ToTokens for ParsedSyscallInputEntry {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let expr = self.0.clone();
        tokens.extend(quote! {#expr});
    }
}

impl Parse for ParsedSyscallInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut entries = Vec::<ParsedSyscallInputEntry>::new();

        while !input.is_empty() {
            let value = input.parse::<syn::Expr>()?;

            entries.push(ParsedSyscallInputEntry(value));

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok (
            Self {
                entries,
            }
        )
    }
}

fn parse_syscall(input: ParsedSyscallInput) -> proc_macro2::TokenStream {

    let entries = input.entries;

    assert!(entries.len() < 6);

    let mut output = proc_macro2::TokenStream::new();

    for entry in entries.iter() {
        output.extend(quote! {
            #entry , 
        });
    }

    for _ in entries.len()..6 {
        output.extend(quote! {
            0 , 
        });
    }

    output
}

#[proc_macro]
pub fn syscall(input: TokenStream) -> TokenStream {
    let output = parse_syscall(parse_macro_input!(input as ParsedSyscallInput));

    quote! {
        crate::syscall::syscall(#output)
    }.into()
}

#[proc_macro]
pub fn syscall_noret(input: TokenStream) -> TokenStream {
    let output = parse_syscall(parse_macro_input!(input as ParsedSyscallInput));

    quote! {
        crate::syscall::syscall_noret(#output)
    }.into()
}
