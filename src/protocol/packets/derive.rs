use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Index, Attribute, Meta, Lit};

#[proc_macro_derive(IntoPacket)]
pub fn derive_into_packet(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let packet_id = match get_packet_id(input.attrs) {
        Ok(packet_id) => packet_id,
        _ => return TokenStream::new(),
    };

    let name = input.ident;

    let generics = add_trait_bounds(input.generics);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let (sum, writes) = sum_and_writes(&input.data);

    let expanded = quote! {
        impl #impl_generics crate::protocol::packets::IntoPacket for #name #ty_generics #where_clase {
            fn into_packet(self) -> crate::protocol::packets::ClientboundPacket {
                let mut data = bytes::BytesMut::with_capacity(#sum);

                #writes

                crate::protocol::packets::ClientboundPacket::new(#packet_id, data);
            }
        }
    };
}

fn get_packet_id(attributes: Vec<Attribute>) -> Result<LitInt, ()> {
    for attribute in attributes {
        if let Ok(Meta::NameValue(name_value)) = attribute.parse_meta() {
            if name_value.path.is_ident("packet_id") {
                if let Lit::Int(v) = name_value.lit {
                    return Ok(v);
                } else {
                    name_value.lit.span().unwrap().error("Packet ID's must be integers").emit();
                    return Err(());
                }
            }
        }
    }
}

// Add a bound `T: HeapSize` to every type parameter T.
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(crate::protocol::data_types::DataType));
        }
    }

    generics
}

fn sum_and_writes(data: &Data) {
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
                    })

                    let writes = fields.named.iter().map(|f| {
                        let name = &f.ident;

                        quote_spanned! {f.span()=>
                            self.#name.write_to(&mut data);
                        }
                    })

                    let sum = quote! {
                        0 + #(+ #sizes)*
                    };
                    let writes = quote! {
                        #(#writes)*
                    }

                    (sum, writes)
                },
                Fields::Unit() => {
                    (quote!(0), quote!())
                },
                Fields::Unnamed(_) => unimplemented!()
            }
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

