extern crate proc_macro;
use proc_macro::{TokenStream};
use proc_macro2::Span;
use syn::{
    AttributeArgs,
    Item, ImplItem, Visibility, Type, Meta, NestedMeta, Attribute, ItemFn, ImplItemMethod, FnDecl, FnArg, Ident, LitStr, Token, Abi, Block,
    VisPublic,
    parse_macro_input,
    parse_quote
};
use quote::quote;
use quote::ToTokens;

fn has_ignore(attrs: &Vec<Attribute>) -> bool {
    for attr in attrs {
        if let Some(path) = attr.path.segments.last() {
            if path.value().ident == "script" {
                if let Ok(Meta::List(parsed)) = attr.parse_meta() {
                    if let NestedMeta::Meta(a) = &parsed.nested[0] {
                        match a {
                            Meta::Word(ref ident) => {
                                if ident == "ignore" {
                                    return true;
                                }
                            },
                            _ => { panic!("Improper script attribute, unrecognized inner detail") },
                        };
                    }
                }
            }
        }
    }

    false
}

fn has_self(function: &FnDecl) -> bool {
    if let Some(first) = function.inputs.first() {
        return match first.value() {
            FnArg::SelfRef(_) => true,
            FnArg::SelfValue(_) => true,
            _ => false,
        };
    }
    false
}

/// For paths, assume we can convert to an opaque pointer.
fn needs_ref(ty: &syn::Type) -> bool {
    false
}

/// For inputs, if the type is a primitive (as defined by cbindgen), we don't
/// do anything. Otherwise, assume we will take it in as a pointer.
fn convert_arg_type(syn::ArgCaptured { ref pat, ref ty, .. }: &syn::ArgCaptured) -> syn::FnArg {
    if ty.clone().into_token_stream().to_string().ends_with("str") {
        parse_quote!(#pat: *const c_char)
    } else {
        if needs_ref(ty) {
            parse_quote!(#pat: *const #ty)
        } else {
            parse_quote!(#pat: #ty)
        }
    }
}

fn generate_fn_ffi(functions: &[ItemFn], item: TokenStream) -> TokenStream {

    let input = proc_macro2::TokenStream::from(item);
    let mut output = quote!{ #input };
    for function in functions {
        let ident = &function.ident;

        let ffi_func_ident = Ident::new(&format!("ffi_{}", ident.to_string()), Span::call_site());
        let mut args = Vec::<proc_macro2::TokenStream>::new();
        let mut caller_args = Vec::<syn::Ident>::new();
        let out = &function.decl.output;

        function.decl.inputs.iter().for_each(|ref arg| {
            match arg {
                FnArg::Captured(ref ac) => {
                    println!("path={:?}", ac.pat);
                    let id = match &ac.pat {
                        syn::Pat::Ident(pi) => {
                            &pi.ident
                        },
                        _ => unimplemented!(),
                    };
                    caller_args.push(id.clone());
                    args.push((ac).into_token_stream());
                },
                _ => {},
            }
        });


        let body: syn::Block = parse_quote!({
            #ident(#(#caller_args),*)
        });

        let ffi_func = ItemFn {
            attrs: Vec::new(),
            vis: Visibility::Public(VisPublic { pub_token : Default::default() }),
            constness: None,
            unsafety: Some(Default::default()),
            asyncness: None,
            abi: Some(Abi { extern_token: Default::default(), name: Some(LitStr::new("C", Span::call_site())) }),
            ident: ffi_func_ident,
            decl: function.decl.clone(),
            block: Box::new(body),
        };
        println!("Outputing: {:?}", ffi_func);

        output = quote!{
            #output

            #ffi_func
        };
    }

    output.into()
}

#[proc_macro_attribute]
pub fn script(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as AttributeArgs);

    let mut accessible = false;

    // Are we an ignore tag? If we are an ignore tag, we just skip ourselves and handle it further down
    if attr.len() < 1 {
        panic!("Improper script attribute, it must have an inner detail")
    }
    if let NestedMeta::Meta(a) = &attr[0] {
        match a {
            Meta::Word(ref ident) => {
                if ident == "ignore" {
                    return item;
                }
                if ident == "accessible" {
                    accessible = true;
                }
            },
            _ => { panic!("Improper script attribute, unrecognized inner detail") },
        };
    }

    let mut scriptable_impl_fn = Vec::new();
    let mut scriptable_fn = Vec::new();

    if accessible {
        let parser_copy = item.clone();
        let parsed = parse_macro_input!(parser_copy as Item);

        match parsed {
            Item::Fn(f) => {
                if ! has_ignore(&f.attrs) && ! has_self(&f.decl) {
                    println!("Got free floating function for serializing: {:?}, {}", f, f.ident.to_string());
                    scriptable_fn.push(f.clone());
                }

            },
            Item::Struct(_) => {},
            Item::Impl(ref data) => {
                for subitem in &data.items {
                    println!("\tsub={:?}", subitem);
                    let path = match *data.self_ty {
                        Type::Path(ref t) => {
                            t.path.segments.iter().map(|s| { s.ident.to_string() }).collect::<Vec<_>>().join("::")
                        },
                        _ => { panic!("LOL") },
                    };
                    match *subitem {
                        ImplItem::Method(ref method) => {
                            match method.vis {
                                Visibility::Public(_) => {
                                    if has_ignore(&method.attrs) {
                                        continue;
                                    }
                                    scriptable_impl_fn.push( ( path, data.clone(), method.clone() ) );
                                },
                                _ => { continue; },
                            }
                        },
                        _ => {},
                    }
                }
            },
            _ => { panic!("Unsupported type for script attribute") }
        }
    }

    // Iterate results and generate them
    let output = generate_fn_ffi(&scriptable_fn, item);

    output
}
