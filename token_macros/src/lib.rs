use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    Ident, LitStr, Result, Token,
};

enum TokenSpec {
    Keyword(Ident),
    Operator(Ident, LitStr),
    OperatorInline((Ident, Ident), (LitStr, LitStr)),
    Span(Ident, LitStr),
    Group((Ident, Ident), LitStr, LitStr)
}

struct TokenSpecs(Vec<TokenSpec>);

impl Parse for TokenSpecs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut specs = Vec::new();

        while !input.is_empty() {
            let variant: Ident = input.parse()?;

            let spec = if input.peek(Token![=>]) {
                input.parse::<Token![=>]>()?;
                let name = variant.to_string();
                let mut inln = name.clone();
                inln.push('I');
                let val: LitStr = input.parse()?;
                let spelling = val.value();
                let mut spelleq = spelling.clone();
                spelleq.push('=');
                TokenSpec::OperatorInline(
                    (
                        Ident::new(&name, Span::call_site()),
                        Ident::new(&inln, Span::call_site())
                    ),
                    (
                        LitStr::new(&spelling, Span::call_site()),
                        LitStr::new(&spelleq, Span::call_site()),
                    )
                )
            } else if input.peek(Token![->]) {
                input.parse::<Token![->]>()?;
                TokenSpec::Operator(variant, input.parse()?)
            } else if input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                TokenSpec::Span(variant, content.parse()?)
            } else if input.peek(syn::token::Bracket) {
                let content;
                syn::bracketed!(content in input);
                let begin = content.parse::<LitStr>()?;
                content.parse::<syn::token::Comma>()?;
                let end = content.parse::<LitStr>()?;
                let name = variant.to_string();
                let mut lname = String::from("L");
                let mut rname = String::from("R");
                lname.push_str(&name);
                rname.push_str(&name);
                TokenSpec::Group(
                    (
                        Ident::new(&lname, Span::call_site()),
                        Ident::new(&rname, Span::call_site())
                    ),
                    begin,
                    end
                )
            } else {
                TokenSpec::Keyword(variant)
            };

            specs.push(spec);

            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        Ok(TokenSpecs(specs))
    }
}

#[proc_macro]
pub fn define_tokens(input: TokenStream) -> TokenStream {
    let TokenSpecs(specs) = parse_macro_input!(input as TokenSpecs);

    let variants = specs.iter().map(|spec| match spec {
        TokenSpec::Keyword(variant) => {
            let spelling = variant.to_string().to_lowercase();
            quote! {
                #[token(#spelling)]
                #variant,
            }
        },
        TokenSpec::OperatorInline((name, inln), (spelling, spelleq)) => quote! {
            #[token(#spelling)]
            #name,
            #[token(#spelleq)]
            #inln,
        },
        TokenSpec::Operator(variant, spelling) => quote! {
            #[token(#spelling)]
            #variant,
        },
        TokenSpec::Span(variant, pattern) => match variant.to_string().as_str() {
            "Unknown" => quote! {
                #[regex(#pattern, priority = 1)]
                #variant(&'src str),
            },
            _ => quote! {
                #[regex(#pattern)]
                #variant(&'src str),
            },
        },
        TokenSpec::Group((lname, rname), begin, end) => quote! {
            #[token(#begin)]
            #lname,
            #[token(#end)]
            #rname,
        }
    });

    quote! {
        #[derive(Logos, Debug, Copy, Clone, PartialEq)]
        #[logos(skip(r"(//.+?\n|/\*.+?\*/|[\s\t\n])", priority = 3))]
        // #[logos(skip(r"[\s\t\n]"))]
        // #[logos(skip(r"(//.+?\n|/\*.+?\*/)"))]
        pub enum Token<'src> {
            #(#variants)*
        }
    }
    .into()
}
