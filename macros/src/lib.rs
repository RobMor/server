extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Attribute, Data, DeriveInput, Fields,
    GenericParam, Generics, Lit, LitInt, Meta,
};

#[proc_macro_derive(Constructor)]
pub fn constructor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    impl_construct_packet_macro(input)
}

fn impl_construct_packet_macro(ast: syn::DeriveInput) -> TokenStream {
    let name = ast.ident;

    let generics = add_trait_bounds(ast.generics);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let (parameters, values) = params_and_values(&ast.data);

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn new(#parameters) -> Self {
                Self {
                    #values
                }
            }
        }
    };

    expanded.into()
}

fn params_and_values(data: &Data) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let mut parameters = Vec::new();
                    let mut values = Vec::new();

                    for f in fields.named.iter() {
                        let ident = &f.ident;
                        let ty = &f.ty;
                        // This is a hack because there is no easy way to support both
                        // SizedDataTypes and DataTypes.
                        parameters.push(quote_spanned! {f.span()=>
                            #ident: #ty
                        });
                        values.push(quote_spanned! {f.span()=>
                            #ident
                        });
                    }

                    let parameters = quote! {
                        #(#parameters),*
                    };
                    let values = quote! {
                        #(#values),*
                    };

                    (parameters.into(), values.into())
                }
                Fields::Unit => unimplemented!(),
                Fields::Unnamed(_) => unimplemented!(),
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

#[proc_macro_derive(IntoPacket, attributes(packet_id))]
pub fn derive_into_packet(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    impl_derive_into_packet(input)
}

fn impl_derive_into_packet(ast: syn::DeriveInput) -> TokenStream {
    let packet_id = match get_packet_id(ast.span(), ast.attrs) {
        Ok(packet_id) => packet_id,
        Err(e) => return e.to_compile_error().into(),
    };

    let name = ast.ident;

    let generics = add_trait_bounds(ast.generics);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let (sum, writes) = sum_and_writes(&ast.data);

    let expanded = quote! {
        impl #impl_generics crate::protocol::packets::IntoPacket for #name #ty_generics #where_clause {
            fn into_packet(self) -> crate::protocol::packets::ClientboundPacket {
                let mut data = bytes::BytesMut::with_capacity(#sum);

                #writes

                crate::protocol::packets::ClientboundPacket::new(#packet_id, data)
            }
        }
    };

    expanded.into()
}

fn get_packet_id(
    global_span: proc_macro2::Span,
    attrs: Vec<syn::Attribute>,
) -> Result<syn::LitInt, syn::Error> {
    for attr in attrs {
        if let Ok(Meta::NameValue(name_value)) = attr.parse_meta() {
            if name_value.path.is_ident("packet_id") {
                if let Lit::Int(v) = name_value.lit {
                    return Ok(v);
                } else {
                    return Err(syn::Error::new(
                        name_value.lit.span(),
                        "Packet IDs must be integers",
                    ));
                }
            }
        }
    }

    Err(syn::Error::new(global_span, "Packet ID must be supplied"))
}

// Add a bound `T: HeapSize` to every type parameter T.
fn add_trait_bounds(mut generics: syn::Generics) -> syn::Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(crate::protocol::data_types::DataType));
        }
    }

    generics
}

fn sum_and_writes(data: &Data) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let sizes = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        // This is a hack because there is no easy way to support both
                        // SizedDataTypes and DataTypes.
                        quote_spanned! {f.span()=>
                            self.#name.size()
                        }
                    });

                    let writes = fields.named.iter().map(|f| {
                        let name = &f.ident;

                        quote_spanned! {f.span()=>
                            self.#name.write_to(&mut data);
                        }
                    });

                    let sum = quote! {
                        0 #(+ #sizes)*
                    };
                    let writes = quote! {
                        #(#writes)*
                    };

                    (sum.into(), writes.into())
                }
                Fields::Unit => (quote!(0), quote!()),
                Fields::Unnamed(_) => unimplemented!(),
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}
