//! The DomainType derive macro implements the tendermint_proto::DomainType trait.
//! This implementation uses the Prost library to convert between Raw types and byte streams.
//!
//! Read more about how to use this macro in the DomainType trait definition.

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

#[proc_macro_derive(DomainType, attributes(rawtype))]
pub fn domaintype(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    expand_domaintype(&input)
}

fn expand_domaintype(input: &syn::DeriveInput) -> TokenStream {
    let ident = &input.ident;

    // Todo: Make this function more robust and easier to read.

    let rawtype_attributes = &input
        .attrs
        .iter()
        .filter(|&attr| attr.path.is_ident("rawtype"))
        .collect::<Vec<&syn::Attribute>>();
    if rawtype_attributes.len() != 1 {
        return syn::Error::new(
            rawtype_attributes.first().span(),
            "exactly one #[rawtype(RawType)] expected",
        )
        .to_compile_error()
        .into();
    }

    let rawtype_tokens = rawtype_attributes[0]
        .tokens
        .clone()
        .into_iter()
        .collect::<Vec<quote::__private::TokenTree>>();
    if rawtype_tokens.len() != 1 {
        return syn::Error::new(rawtype_attributes[0].span(), "#[rawtype(RawType)] expected")
            .to_compile_error()
            .into();
    }

    let rawtype = match &rawtype_tokens[0] {
        proc_macro2::TokenTree::Group(group) => group.stream(),
        _ => {
            return syn::Error::new(
                rawtype_tokens[0].span(),
                "#[rawtype(RawType)] group expected",
            )
            .to_compile_error()
            .into()
        }
    };

    let gen = quote! {
        impl ::tendermint_proto::DomainType<#rawtype> for #ident {

            fn encode<B: ::tendermint_proto::bytes::BufMut>(self, buf: &mut B) -> ::std::result::Result<(), ::tendermint_proto::Error> {
                use ::tendermint_proto::prost::Message;
                #rawtype::from(self).encode(buf).map_err(|e| ::tendermint_proto::Kind::EncodeMessage.context(e).into())
            }

            fn encode_length_delimited<B: ::tendermint_proto::bytes::BufMut>(self, buf: &mut B) -> ::std::result::Result<(), ::tendermint_proto::Error> {
                use ::tendermint_proto::prost::Message;
                #rawtype::from(self).encode_length_delimited(buf).map_err(|e| ::tendermint_proto::Kind::EncodeMessage.context(e).into())
            }

            fn decode<B: ::tendermint_proto::bytes::Buf>(buf: B) -> Result<Self, ::tendermint_proto::Error> {
                use ::tendermint_proto::prost::Message;
                #rawtype::decode(buf).map_or_else(
                    |e| ::std::result::Result::Err(::tendermint_proto::Kind::DecodeMessage.context(e).into()),
                    |t| Self::try_from(t).map_err(|e| ::tendermint_proto::Kind::TryIntoDomainType.context(e).into())
                )
            }

            fn decode_length_delimited<B: ::tendermint_proto::bytes::Buf>(buf: B) -> Result<Self, ::tendermint_proto::Error> {
                use ::tendermint_proto::prost::Message;
                #rawtype::decode_length_delimited(buf).map_or_else(
                    |e| ::std::result::Result::Err(::tendermint_proto::Kind::DecodeMessage.context(e).into()),
                    |t| Self::try_from(t).map_err(|e| ::tendermint_proto::Kind::TryIntoDomainType.context(e).into())
                )
            }

            fn encoded_len(self) -> usize {
                use ::tendermint_proto::prost::Message;
                #rawtype::from(self).encoded_len()
            }

        }
    };
    gen.into()
}
