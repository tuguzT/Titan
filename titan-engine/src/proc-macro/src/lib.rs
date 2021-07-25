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
                static ONCE: ::std::sync::Once = ::std::sync::Once::new();
                static mut SLOTMAP: Option<::std::sync::RwLock<::slotmap::SlotMap<Key, #name>>> = None;
                unsafe {
                    ONCE.call_once(|| {
                        SLOTMAP = Some(::std::sync::RwLock::new(::slotmap::SlotMap::with_key()));
                    });
                    SLOTMAP.as_ref().unwrap()
                }
            }
        }
    };
    TokenStream::from(gen)
}
