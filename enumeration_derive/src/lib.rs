use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
#[allow(clippy::wildcard_imports)]
use syn::*;

#[allow(dead_code)]
#[repr(C)]
enum SizedEnum {
    A,
    B,
}

/// Probably 32.
const C_ENUM_BITS: usize = std::mem::size_of::<SizedEnum>() * 8;

#[allow(clippy::too_many_lines)]
#[proc_macro_derive(Enum)]
pub fn derive_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    assert!(!input.variants.is_empty(), "type must not be empty");

    if let Some(variant) = input.variants.iter().find(|x| x.discriminant.is_some()) {
        return TokenStream::from(
            syn::Error::new_spanned(variant, "manual discriminants are unsupported")
                .into_compile_error(),
        );
    }

    let size = input.variants.len();

    let Some(rep) = rep_for_size(size + 1) else {
        panic!("too many variants");
    };

    let min_bound = &input.variants.first().unwrap().ident;
    let max_bound = &input.variants.last().unwrap().ident;

    #[cfg(feature = "inline")]
    let inline = quote!(#[inline]);
    #[cfg(not(feature = "inline"))]
    let inline = quote!();

    let prologue = quote! {
        type Rep = #rep;
        const SIZE: usize = #size;
        const MIN: Self = #name::#min_bound;
        const MAX: Self = #name::#max_bound;
    };

    let idx = match find_repr(&input.attrs) {
        None if size > 2 => Some(Ident::new("u8", Span::call_site())),
        idx => idx,
    };

    let expanded = if let Some(idx) = idx {
        let size_assertion_error = format!("unable to find a suitable repr\nspecify #[repr(u8)] or another integer type\n(guessed {idx})");

        quote! {
            const _: () = assert!(
                std::mem::size_of::<#name>() == std::mem::size_of::<#idx>(),
                #size_assertion_error,
            );

            impl #impl_generics Enum for #name #ty_generics #where_clause {
                #prologue

                #inline
                fn succ(self) -> Option<Self> {
                    if self == #name::#max_bound {
                        None
                    } else {
                        Some(unsafe { std::mem::transmute(self as #idx + 1) })
                    }
                }

                #inline
                fn pred(self) -> Option<Self> {
                    if self == #name::#min_bound {
                        None
                    } else {
                        Some(unsafe { std::mem::transmute(self as #idx - 1) })
                    }
                }

                #inline
                fn bit(self) -> Self::Rep {
                    1 << (self as #idx)
                }

                #inline
                fn index(self) -> usize {
                    self as usize
                }

                #inline
                fn from_index(i: usize) -> Option<Self> {
                    if i < #size {
                        Some(unsafe { std::mem::transmute(i as #idx) })
                    } else {
                        None
                    }
                }
            }

            impl #impl_generics #name #ty_generics #where_clause {
                #[doc(hidden)]
                #inline
                pub const fn bit(self) -> #rep {
                    1 << (self as #idx)
                }
            }
        }
    } else if size == 1 {
        quote! {
            impl #impl_generics Enum for #name #ty_generics #where_clause {
                #prologue

                #inline
                fn succ(self) -> Option<Self> {
                    None
                }

                #inline
                fn pred(self) -> Option<Self> {
                    None
                }

                #inline
                fn bit(self) -> Self::Rep {
                    0
                }

                #inline
                fn index(self) -> usize {
                    0
                }

                #inline
                fn from_index(i: usize) -> Option<Self> {
                    match i {
                        0 => Some(#name::#min_bound),
                        _ => None,
                    }
                }
            }

            impl #impl_generics #name #ty_generics #where_clause {
                #[doc(hidden)]
                #inline
                pub const fn bit(self) -> #rep {
                    0
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics Enum for #name #ty_generics #where_clause {
                #prologue

                #inline
                fn succ(self) -> Option<Self> {
                    match self {
                        #name::#max_bound => None,
                        #name::#min_bound => Some(#name::#max_bound)
                    }
                }

                #inline
                fn pred(self) -> Option<Self> {
                    match self {
                        #name::#min_bound => None,
                        #name::#max_bound => Some(#name::#min_bound)
                    }
                }

                #inline
                fn bit(self) -> Self::Rep {
                    self as #rep
                }

                #inline
                fn index(self) -> usize {
                    self as usize
                }

                #inline
                fn from_index(i: usize) -> Option<Self> {
                    match i {
                        0 => Some(#name::#min_bound),
                        1 => Some(#name::#max_bound),
                        _ => None,
                    }
                }
            }

            impl #impl_generics #name #ty_generics #where_clause {
                #[doc(hidden)]
                #inline
                pub const fn bit(self) -> #rep {
                    self as #rep
                }
            }
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

fn find_repr(attrs: &[Attribute]) -> Option<Ident> {
    let repr = attrs
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
        .next()?;

    match repr.to_string().as_str() {
        "C" => Some(Ident::new(&format!("u{C_ENUM_BITS}"), Span::call_site())),
        "Rust" => None,
        _ => Some(repr),
    }
}
