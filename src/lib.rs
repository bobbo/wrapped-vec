#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{Ident, DeriveInput};

struct Idents {
    item: Ident,
    collection: Ident
}

impl Idents {

    fn new(ast: &DeriveInput) -> Result<Idents, String> {
        let collection_name = attr_string_val(ast, "CollectionName").expect("Need [CollectionName=\"...\"]");

        Ok(Idents{
            item: ast.ident.clone(),
            collection: Ident::from(collection_name)
        })
    }

    fn as_parts(&self) -> (&Ident, &Ident) {
        (&self.item, &self.collection)
    }

}

struct Docs {
    wrapper: String,
    new: String,
    is_empty: String,
    len: String,
    iter: String
}

macro_rules! doc_attr {
    ($ast:ident, $attr:expr, $default:expr) => (
        attr_string_val($ast, $attr).unwrap_or($default);
    )
}

impl Docs {

    fn new(ast: &DeriveInput, idents: &Idents) -> Docs {
        let wrapper = doc_attr!(ast, "CollectionDoc", format!("A collection of {}s", idents.item));
        let new = doc_attr!(ast, "NewDoc", format!("Creates a new, empty {}", idents.collection));
        let is_empty = doc_attr!(ast, "IsEmptyDoc", format!("Returns true if the {} contains no {}s", idents.collection, idents.item));
        let len = doc_attr!(ast, "LenDoc", format!("Returns the number of {}s in the {}", idents.item, idents.collection));
        let iter = doc_attr!(ast, "IterDoc", format!("Returns an iterator over the {}", idents.collection));

        Docs {
            wrapper: wrapper,
            new: new,
            is_empty: is_empty,
            len: len,
            iter: iter
        }
    }

    fn as_parts(&self) -> (&String, &String, &String, &String, &String) {
        (&self.wrapper, &self.new, &self.is_empty, &self.len, &self.iter)
    }

}

#[proc_macro_derive(WrappedVec, 
    attributes(
        CollectionName,
        CollectionDoc,
        NewDoc,
        IsEmptyDoc,
        LenDoc,
        IterDoc
    )
)]
pub fn wrapped_vec(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();

    match impl_wrapped_vec(&ast) {
        Ok(gen) => {
            gen.parse().unwrap()
        },
        Err(err) => {
            panic!(err);
        }
    }
}

fn impl_wrapped_vec(ast: &DeriveInput) -> Result<quote::Tokens, String> {
    let idents = Idents::new(ast)?;
    let docs = Docs::new(ast, &idents);

    Ok(generate_wrapped_vec(&idents, &docs))
}

fn generate_wrapped_vec(idents: &Idents, docs: &Docs) -> quote::Tokens {
    let (item_ident, collection_ident) = idents.as_parts();
    let (collection_doc, new_doc, is_empty_doc, len_doc, iter_doc) = docs.as_parts();

    quote! {
        #[doc=#collection_doc]
        pub struct #collection_ident(Vec<#item_ident>);

        impl ::std::iter::FromIterator<#item_ident> for #collection_ident {
            fn from_iter<I: IntoIterator<Item=#item_ident>>(iter: I) -> Self {
                let mut inner = vec![];
                inner.extend(iter);
                #collection_ident(inner)
            }
        }

        impl From<Vec<#item_ident>> for #collection_ident {

            fn from(ids: Vec<#item_ident>) -> #collection_ident {
                let mut new = #collection_ident::new();
                new.extend(ids);
                new
            }

        }

        impl IntoIterator for #collection_ident {
            type Item = #item_ident;
            type IntoIter = ::std::vec::IntoIter<#item_ident>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        impl<'a> IntoIterator for &'a #collection_ident {
            type Item = &'a #item_ident;
            type IntoIter = ::std::slice::Iter<'a, #item_ident>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.iter()
            }
        }

        impl Extend<#item_ident> for #collection_ident {
            fn extend<T: IntoIterator<Item=#item_ident>>(&mut self, iter: T) {
                self.0.extend(iter);
            }
        }

        impl #collection_ident {

            #[doc=#new_doc]
            pub fn new() -> #collection_ident {
                #collection_ident(vec![])
            }

            #[doc=#is_empty_doc]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            #[doc=#len_doc]
            pub fn len(&self) -> usize {
                self.0.len()
            }

            #[doc=#iter_doc]
            pub fn iter<'a>(&'a self) -> ::std::slice::Iter<'a, #item_ident> {
                self.into_iter()
            }

        }
    }
}

fn attr_string_val(ast: &syn::DeriveInput, attr_name: &'static str) -> Option<String> {
    if let Some(ref a) = ast.attrs.iter().find(|a| a.name() == attr_name) {
        if let syn::MetaItem::NameValue(_, syn::Lit::Str(ref val, _)) = a.value {
            return Some(val.clone())
        }
        else {
            return None
        }
    } else {
        return None
    }
}
