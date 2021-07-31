use proc_macro::TokenStream;

use syn::{Data, DeriveInput, Fields};

#[proc_macro_derive(SlotMappable, attributes(key))]
pub fn slot_mappable_macro_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let name = &ast.ident;
    let key = {
        let mut keys = Vec::new();
        match &ast.data {
            Data::Struct(data) => match &data.fields {
                Fields::Named(fields_named) => {
                    for field in fields_named.named.iter() {
                        for attr in field.attrs.iter() {
                            let path = &attr.path;
                            if *path.get_ident().unwrap() == "key" {
                                keys.push((field.ident.as_ref().unwrap(), &field.ty))
                            }
                        }
                    }
                }
                _ => panic!("struct must have a field annotated with `key` attribute"),
            },
            _ => panic!("macro applicable only for struct"),
        }
        if keys.len() > 1 {
            panic!("there must be unique `key` attribute");
        }
        *keys
            .get(0)
            .expect("struct must have a field annotated with `key` attribute")
    };
    let (key_ident, key_ty) = key;

    let gen = quote::quote! {
        impl SlotMappable for #name {
            type Key = #key_ty;

            fn key(&self) -> Self::Key {
                self.#key_ident
            }

            fn slotmap() -> &'static ::std::sync::RwLock<::slotmap::SlotMap<Self::Key, Self>> {
                ::lazy_static::lazy_static! {
                    static ref SLOTMAP: ::std::sync::RwLock<::slotmap::SlotMap<#key_ty, #name>> =
                        ::std::sync::RwLock::new(::slotmap::SlotMap::with_key());
                }
                &*SLOTMAP
            }
        }
    };
    TokenStream::from(gen)
}
