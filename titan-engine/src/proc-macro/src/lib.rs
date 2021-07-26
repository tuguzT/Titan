use proc_macro::TokenStream;

#[proc_macro_derive(SlotMappable)]
pub fn slot_mappable_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_slot_mappable_macro_derive(&ast)
}

fn impl_slot_mappable_macro_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote::quote! {
        impl SlotMappable for #name {
            type Key = Key;

            fn key(&self) -> Self::Key {
                self.key
            }

            fn slotmap() -> &'static ::std::sync::RwLock<::slotmap::SlotMap<Self::Key, Self>>
            where
                Self: Sized + Send + Sync,
            {
                ::lazy_static::lazy_static! {
                    static ref SLOTMAP: ::std::sync::RwLock<::slotmap::SlotMap<Key, #name>> =
                        ::std::sync::RwLock::new(::slotmap::SlotMap::with_key());
                }
                &*SLOTMAP
            }
        }
    };
    TokenStream::from(gen)
}
