pub mod element;
pub mod expand_enum;
pub mod expand_struct;
pub mod implement_serializer;
pub mod label;
pub mod namespace;

use crate::common::YaSerdeAttribute;
use proc_macro2::TokenStream;
use quote::quote;

pub fn expand_derive_serialize(ast: &syn::DeriveInput) -> Result<TokenStream, String> {
  let name = &ast.ident;
  let attrs = &ast.attrs;
  let data = &ast.data;
  let generics = &ast.generics;

  let root_attributes = YaSerdeAttribute::parse(attrs);

  let root_name = format!(
    "{}{}",
    root_attributes.prefix_namespace(),
    root_attributes.xml_element_name(name)
  );

  let impl_block = match *data {
    syn::Data::Struct(ref data_struct) => {
      expand_struct::serialize(data_struct, name, &root_name, &root_attributes, generics)
    }
    syn::Data::Enum(ref data_enum) => {
      expand_enum::serialize(data_enum, name, &root_name, &root_attributes, generics)
    }
    syn::Data::Union(ref _data_union) => unimplemented!(),
  };

  Ok(quote! {
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
      use ::std::str::FromStr as _;

      #impl_block
    };
  })
}
