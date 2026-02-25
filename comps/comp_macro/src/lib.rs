use proc_macro2::TokenStream as TokenStream2;
use quote::{quote,ToTokens};
use syn::{
    parse::{Parse, ParseStream}, 
    parse_macro_input,
    Expr, Pat, Token,    
};

struct Comp {
    mapping: Mapping,
    for_if_clause: ForIfClause,
}

impl Parse for Comp {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            mapping: input.parse()?,
            for_if_clause: input.parse()?,
        })
    }
}

impl ToTokens for Comp {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Mapping(mapping) = &self.mapping;
        let ForIfClause {
            pattern,
            expression,
            condition
        } = &self.for_if_clause;

        let conditions = condition.iter().map(|c| {
            let inner = &c.0;
            quote! { #inner }
        });

        tokens.extend(quote! {
            core::iter::IntoIterator::into_iter(#expression).filter_map(move |#pattern| {
                (true #(&& (#conditions))*).then(|| #mapping)
            })
        })
    }
}

struct Mapping(Expr);

impl Parse for Mapping {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse()?))
    }
}

impl ToTokens for Mapping {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.0.to_tokens(tokens);
    }
}

struct ForIfClause {
    pattern: Pattern,
    expression: Expr,
    condition: Vec<Condition>,
}

impl Parse for ForIfClause {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        _ = input.parse::<Token![for]>()?;
        let pattern = input.parse()?;
        _ = input.parse::<Token![in]>()?;
        let expression = input.parse()?;
        let condition = parse_zero_or_more(input);
        Ok(Self {
            pattern,
            expression,
            condition,
        })
    }
}

fn parse_zero_or_more<T: Parse>(input: ParseStream) -> Vec<T> {
    let mut result = Vec::new();
    while let Ok(item) = input.parse() {
        result.push(item);
    }
    result
}

struct Pattern(Pat);

impl Parse for Pattern {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Pat::parse_single(input).map(Self)
    }
}

impl ToTokens for Pattern {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.0.to_tokens(tokens);
    }
}

struct Condition(Expr);

impl Parse for Condition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        _ = input.parse::<syn::Token![if]>()?;
        input.parse().map(Self)
    }
}

impl ToTokens for Condition {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.0.to_tokens(tokens);
    }
}

#[proc_macro]
pub fn comp(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let c = parse_macro_input!(input as Comp);
    quote! { #c }.into()
}