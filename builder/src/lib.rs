use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let vis = &input.vis;
    let ident = &input.ident;
    let builder_ident = format_ident!("{}Builder", input.ident);

    let generated = quote! {

        impl #ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {}
            }
        }

        #vis struct #builder_ident {
        }
    };
    generated.into()
}
