use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, AngleBracketedGenericArguments, Data, DeriveInput, Fields, GenericArgument,
    Lit, LitStr, Path, PathArguments, PathSegment, Type, TypePath,
};

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
    let builder_fields: Vec<_> = fields
        .named
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            match try_get_option_generics(&field.ty) {
                Ok(inner_ty) => quote! { #ident: Option<#inner_ty> },
                Err(original_ty) => quote! { #ident: Option<#original_ty> },
            }
        })
        .collect();

    let builder_init: Vec<_> = fields
        .named
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            quote! { #ident: None }
        })
        .collect();

    let setter_methods: Vec<_> = fields
        .named
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let ty = match try_get_option_generics(&field.ty) {
                Ok(inner_ty) => inner_ty,
                Err(original_ty) => original_ty,
            };
            quote! {
                pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                    self.#ident = Some(#ident);
                    self
                }
            }
        })
        .collect();

    let field_setters: Vec<_> = fields.named.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let error_msg = LitStr::new(&format!("field {} is not set", ident), ident.span());
        let literal = Lit::Str(error_msg);
        if try_get_option_generics(&field.ty).is_ok() {
            quote! { #ident: self.#ident.clone() }
        } else {
            quote! { #ident: self.#ident.clone().ok_or(Box::<dyn std::error::Error>::from(#literal.to_string()))? }
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

            pub fn build(&mut self) -> Result<#ident, Box<dyn std::error::Error>> {
                Ok(#ident {
                    #(#field_setters),*
                })
            }
        }
    };
    generated.into()
}

fn try_get_option_generics(ty: &Type) -> Result<&Type, &Type> {
    if let Type::Path(TypePath {
        qself: None,
        path: Path {
            leading_colon: None,
            segments,
        },
    }) = ty
    {
        if segments.len() == 1 {
            let segment = segments.first().unwrap();
            if let PathSegment {
                ident,
                arguments:
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }),
            } = segment
            {
                if ident == "Option" && args.len() == 1 {
                    if let Some(GenericArgument::Type(inner_ty)) = args.first() {
                        return Ok(inner_ty);
                    }
                }
            }
        }
    }
    Err(ty)
}
