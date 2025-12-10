pub mod build_default_value;
pub mod expand_enum;
pub mod expand_struct;

use crate::common::YaSerdeAttribute;
use proc_macro2::TokenStream;
use quote::quote;

pub fn expand_derive_deserialize(ast: &syn::DeriveInput) -> Result<TokenStream, String> {
  let name = &ast.ident;
  let attrs = &ast.attrs;
  let data = &ast.data;
  let generics = &ast.generics;

  let root_attributes = YaSerdeAttribute::from(attrs);

  let root_name = root_attributes.xml_element_name(name);

  let impl_block = match *data {
    syn::Data::Struct(ref data_struct) => {
      expand_struct::parse(data_struct, name, &root_name, &root_attributes, generics)
    }
    syn::Data::Enum(ref data_enum) => {
      expand_enum::parse(data_enum, name, &root_name, &root_attributes, generics)
    }
    syn::Data::Union(ref _data_union) => unimplemented!(),
  };

  Ok(quote! {
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
      use ::std::str::FromStr as _;
      use ::yaserde::Visitor as _;

      #impl_block
    };
  })
}
