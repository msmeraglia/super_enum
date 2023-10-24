use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(SuperEnum)]
pub fn super_enum(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    if let syn::Data::Enum(data_enum) = &ast.data {
        let enum_name = &ast.ident;
        let variant_names = data_enum.variants.iter().map(|variant| &variant.ident);
        let variant_str_names = data_enum
            .variants
            .iter()
            .map(|variant| variant.ident.to_string());
        let enum_count = variant_names.len();
        let super_fn = quote! {
            impl #enum_name {
                pub fn all() -> &'static [#enum_name; #enum_count] {
                    static ALL: [#enum_name; #enum_count] = [
                        #(#enum_name::#variant_names),*
                    ];
                    &ALL
                }

                pub fn as_str_array() -> &'static [&'static str;#enum_count] {
                    static STRS: [&'static str; #enum_count] = [
                        #(#variant_str_names),*
                    ];
                    &STRS
                }
            }
        };
        super_fn.into()
    } else {
        TokenStream::from(
            syn::Error::new_spanned(ast, "SuperEnum can only be used with enums.")
                .to_compile_error(),
        )
    }
}
