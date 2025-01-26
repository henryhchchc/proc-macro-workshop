use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let vis = &input.vis;
    let ident = &input.ident;
    let builder_ident = format_ident!("{}Builder", input.ident);

    let Data::Struct(strukt) = input.data else {
        panic!("Builder derive only supports struct");
    };
    let Fields::Named(fields) = strukt.fields else {
        panic!("Builder derive only supports named fields");
    };
    let builder_fields: Vec<_> = fields.named.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        quote! { #ident: Option<#ty> }
    }).collect();

    let builder_init: Vec<_> = fields.named.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        quote! { #ident: None }
    }).collect();

    let setter_methods: Vec<_> = fields.named.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        quote! {
            pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                self.#ident = Some(#ident);
                self
            }
        }
    }).collect();

    let generated = quote! {

        impl #ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#builder_init),*
                }
            }
        }

        #vis struct #builder_ident {
            #(#builder_fields),*
        }

        impl #builder_ident {
            #(#setter_methods)*
        }
    };
    generated.into()
}
