use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::*;

#[proc_macro_derive(Enum)]
pub fn derive_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    if input.variants.is_empty() {
        panic!("type must not be empty");
    }

    if let Some(variant) = input.variants.iter().find(|x| x.discriminant.is_some()) {
        return TokenStream::from(
            syn::Error::new_spanned(variant, "manual discriminants are unsupported")
                .into_compile_error(),
        );
    }

    let size = input.variants.len();

    let rep = match rep_for_size(size) {
        Some(rep) => rep,
        None => panic!("Too many variants"),
    };

    let idx = input
        .attrs
        .iter()
        .map(Attribute::parse_meta)
        .filter_map(Result::ok)
        .filter(|x| x.path().is_ident("repr"))
        .filter_map(|x| match x {
            Meta::List(meta) => Some(meta.nested),
            _ => None,
        })
        .flat_map(IntoIterator::into_iter)
        .filter_map(|x| match x {
            NestedMeta::Meta(Meta::Path(path)) => Some(path.segments),
            _ => None,
        })
        .flat_map(IntoIterator::into_iter)
        .map(|x| x.ident)
        .next()
        .unwrap_or_else(|| Ident::new("u8", Span::call_site()));

    let min_bound = &input.variants.first().unwrap().ident;
    let max_bound = &input.variants.last().unwrap().ident;

    let expanded = quote! {
        impl #impl_generics Enum for #name #ty_generics #where_clause {
            type Rep = #rep;
            const SIZE: usize = #size;
            const MIN: Self = #name::#min_bound;
            const MAX: Self = #name::#max_bound;

            fn succ(self) -> Option<Self> {
                if self == #name::#max_bound {
                    None
                } else {
                    Some(unsafe { std::mem::transmute(self as #idx + 1) })
                }
            }

            fn pred(self) -> Option<Self> {
                if self == #name::#min_bound {
                    None
                } else {
                    Some(unsafe { std::mem::transmute(self as #idx - 1) })
                }
            }

            fn bit(self) -> Self::Rep {
                1 << (self as #idx)
            }

            fn index(self) -> usize {
                self as usize
            }

            fn from_index(i: usize) -> Option<Self> {
                if i < #size {
                    Some(unsafe { std::mem::transmute(i as #idx) })
                } else {
                    None
                }
            }
        }

        impl #impl_generics #name #ty_generics #where_clause {
            pub const fn bit(self) -> #rep {
                1 << (self as #idx)
            }
        }
    };

    let option_rep = match rep_for_size(size + 1) {
        Some(option_rep) => option_rep,
        None => return TokenStream::from(expanded),
    };

    let expanded = quote! {
        #expanded

        impl #impl_generics crate::optionable::OptionableEnum for #name #ty_generics #where_clause {
            type RepForOptional = #option_rep;
        }
    };

    TokenStream::from(expanded)
}

fn rep_for_size(size: usize) -> Option<proc_macro2::TokenStream> {
    if size <= 8 {
        Some(quote!(u8))
    } else if size <= 16 {
        Some(quote!(u16))
    } else if size <= 32 {
        Some(quote!(u32))
    } else if size <= 64 {
        Some(quote!(u64))
    } else if size <= 128 {
        Some(quote!(u128))
    } else {
        None
    }
}
