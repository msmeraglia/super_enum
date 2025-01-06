use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(SuperEnum)]
pub fn super_enum(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    if let syn::Data::Enum(data_enum) = &ast.data {
        let enum_name = &ast.ident;

        let variant_count = data_enum.variants.len();
        let mut to_u64_match_arms = Vec::new();
        let mut to_usize_match_arms = Vec::new();
        let mut from_usize_match_arms = Vec::new();
        let mut variant_str_names = Vec::new();

        for (i, variant) in data_enum.variants.iter().enumerate() {
            let variant_name = &variant.ident;
            let formatted_str = variant_name
                .to_string()
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    if i > 0 && c.is_uppercase() {
                        format!(" {}", c)
                    } else {
                        c.to_string()
                    }
                })
                .collect::<String>();

            variant_str_names.push(formatted_str);
            match &variant.fields {
                syn::Fields::Named(fields) => {
                    let field_names = fields.named.iter().map(|field| &field.ident);
                    to_u64_match_arms.push(quote! {
                        #enum_name::#variant_name { .. } => #i as u64,
                    });
                    to_usize_match_arms.push(quote! {
                        #enum_name::#variant_name { .. } => #i,
                    });
                    from_usize_match_arms.push(quote! {
                        #i => #enum_name::#variant_name { #( #field_names: Default::default() ),* } ,
                    });
                }
                syn::Fields::Unnamed(fields) => {
                    let field_names =
                        (0..fields.unnamed.len()).map(|_| quote! { Default::default() });
                    to_u64_match_arms.push(quote! {
                        #enum_name::#variant_name(..) => #i as u64,
                    });
                    to_usize_match_arms.push(quote! {
                        #enum_name::#variant_name(..) => #i,
                    });
                    from_usize_match_arms.push(quote! {
                        #i => #enum_name::#variant_name( #( #field_names ),* ),
                    });
                }
                syn::Fields::Unit => {
                    to_u64_match_arms.push(quote! {
                        #enum_name::#variant_name => #i as u64,
                    });
                    to_usize_match_arms.push(quote! {
                        #enum_name::#variant_name => #i,
                    });
                    from_usize_match_arms.push(quote! {
                        #i => #enum_name::#variant_name,
                    });
                }
            }
        }

        let impl_tokens = quote! {
            impl #enum_name {
                pub fn all() -> Vec<#enum_name> {
                    let mut all = Vec::with_capacity(#variant_count);
                    for i in 0..#variant_count {
                        all.push(Self::from(i))
                    }
                    all
                }

                pub const fn as_str_array() -> &'static [&'static str;#variant_count] {
                    const STRS: [&'static str; #variant_count] = [
                        #(#variant_str_names),*
                    ];
                    &STRS
                }

                pub fn as_str(&self) -> &'static str {
                    let id: usize  = (*self).into();
                    &#enum_name::as_str_array()[id]
                }

                pub const fn len() -> usize {
                    #variant_count
                }
            }

            impl Into<usize> for #enum_name {
                fn into(self) -> usize {
                    self as usize
                }
            }

            impl From<usize> for #enum_name {
                fn from(value: usize) -> Self {
                    match value {
                        #( #from_usize_match_arms )*
                        _ => panic!("Invalid enum variant index: {}", value),
                    }
                }
            }

            impl Into<u64> for #enum_name {
                fn into(self) -> u64 {
                    self as u64
                }
            }
        };
        impl_tokens.into()
    } else {
        TokenStream::from(
            syn::Error::new_spanned(ast, "SuperEnum can only be used with enums.")
                .to_compile_error(),
        )
    }
}
