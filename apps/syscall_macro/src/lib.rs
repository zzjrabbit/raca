use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, FnArg, Signature, Token, Visibility};

/*struct ParsedSyscallInput {
    vis: syn::Visibility,
    attr: Vec<syn::Attribute>,
    fn_name: syn::Ident,
    args: Vec<(syn::Ident, syn::Type)>,
    ret: syn::Type,
}

impl ToTokens for ParsedSyscallInputEntry {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let expr = self.0.clone();
        tokens.extend(quote! {#expr});
    }
}

impl Parse for ParsedSyscallInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attrs = Vec::new();
        while !input.is_empty() {
            let Ok(attr) = input.parse::<syn::Attribute>() else {
                break;
            };
            attrs.push(attr);
        }

        let vis = input.parse()?;
        input.parse::<Token![fn]>()?;
        let fn_name = input.parse()?;
        input.parse::<syn::token::Paren>()?;
        let mut args = Vec::new();
        while !input.is_empty() {
            let arg = input.parse::<syn::Ident>()?;
            input.parse::<Token![:]>()?;
            let ty = input.parse::<syn::Type>()?;
            args.push((arg, ty));
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        if !input.is_empty() {
            input.parse::<Token![,]>()?;
            assert!(input.is_empty());
        }

        // TODO: parse return type
        let ret = syn::parse::<syn::Type>()?;

        input.parse::<Token![)]>()?;
        input.parse::<Token![->]>()?;
        let ret = input.parse::<syn::Type>()?;

        Ok(Self {
            vis,

            ret,
        })
    }
}

fn parse_syscall(input: ParsedSyscallInput) -> proc_macro2::TokenStream {
    let entries = input.entries;

    assert!(entries.len() <= 6);

    let mut output = proc_macro2::TokenStream::new();

    for entry in entries.iter() {
        output.extend(quote! {
            #entry ,
        });
    }

    output
}*/

struct Syscall {
    signature: Signature,
    id: syn::Expr,
    vis: Visibility,
}

impl Parse for Syscall {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let id = input.parse::<syn::Expr>().expect("id");
        input.parse::<Token![,]>().expect("comma");
        let vis = input.parse::<Visibility>().expect("visibility");
        let signature = input.parse::<Signature>().expect("signature");
        Ok(Syscall { signature, id, vis })
    }
}

#[proc_macro]
pub fn syscall(input: TokenStream) -> TokenStream {
    let Syscall { signature, id, vis } = syn::parse_macro_input!(input as Syscall);
    let ret = signature.output.clone().into_token_stream();

    let syscall_function = {
        let mut syscall_function: Signature =
            syn::parse_quote! {extern "C" fn syscall(id: u64) #ret};

        for i in 0..signature.inputs.len() {
            let arg_name = syn::parse_str::<syn::Ident>(format!("arg{}", i).as_str()).unwrap();
            let arg: FnArg = syn::parse_quote! {#arg_name: usize};
            syscall_function.inputs.push(arg);
        }

        quote! {
            #[naked]
            #[allow(improper_ctypes)]
            #syscall_function {
                unsafe {
                    core::arch::naked_asm!(
                        "mov r10,rcx",
                        "syscall",
                        "ret",
                    )
                }
            }
        }
    };

    let calling_args = {
        let mut token_stream = proc_macro2::TokenStream::new();
        for i in 0..signature.inputs.len() {
            let arg = signature.inputs[i].clone();
            let name = match arg {
                FnArg::Receiver(_) => panic!("syscall cannot have receiver"),
                FnArg::Typed(arg) => arg.pat.clone(),
            };
            token_stream.extend(quote! {
                #name as usize,
            });
        }
        token_stream
    };

    quote! {
        #vis #signature {
            #syscall_function
            syscall(#id as u64, #calling_args)
        }
    }
    .into()
}

#[proc_macro]
pub fn syscall_noret(input: TokenStream) -> TokenStream {
    let Syscall { signature, id, vis } = syn::parse_macro_input!(input as Syscall);
    let ret = signature.output.clone().into_token_stream();

    let syscall_function = {
        let mut syscall_function: Signature =
            syn::parse_quote! {extern "C" fn syscall(id: u64) #ret};

        for i in 0..signature.inputs.len() {
            let arg_name = syn::parse_str::<syn::Ident>(format!("arg{}", i).as_str()).unwrap();
            let arg: FnArg = syn::parse_quote! {#arg_name: usize};
            syscall_function.inputs.push(arg);
        }

        quote! {
            #[naked]
            #[allow(improper_ctypes)]
            #syscall_function {
                unsafe {
                    core::arch::naked_asm!(
                        "mov r10,rcx",
                        "syscall",
                        "_b:",
                        "jmp _b",
                    )
                }
            }
        }
    };

    let calling_args = {
        let mut token_stream = proc_macro2::TokenStream::new();
        for i in 0..signature.inputs.len() {
            let arg = signature.inputs[i].clone();
            let name = match arg {
                FnArg::Receiver(_) => panic!("syscall cannot have receiver"),
                FnArg::Typed(arg) => arg.pat.clone(),
            };
            token_stream.extend(quote! {
                #name,
            });
        }
        token_stream
    };

    quote! {
        #vis #signature {
            #syscall_function
            syscall(#id as u64, #calling_args)
        }
    }
    .into()
}
