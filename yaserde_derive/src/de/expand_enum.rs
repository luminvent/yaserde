use crate::common::{Field, YaSerdeAttribute, YaSerdeField};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DataEnum, Fields, Generics, Ident};

pub fn parse(
  data_enum: &DataEnum,
  name: &Ident,
  root: &str,
  root_attributes: &YaSerdeAttribute,
  generics: &Generics,
) -> syn::Result<TokenStream> {
  let namespaces_matching = root_attributes.get_namespace_matching(
    &None,
    quote!(enum_namespace),
    quote!(named_element),
    true,
  );

  let match_to_enum: TokenStream = data_enum
    .variants
    .iter()
    .map(|variant| parse_variant(variant, name))
    .collect::<syn::Result<_>>()?;

  let flatten = root_attributes.flatten;
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

  let element_name = if let Some(tag) = &root_attributes.tag {
    quote! {
      attributes
        .iter()
        .find(|attr| attr.name.local_name.as_str() == #tag)
        .ok_or_else(|| ::yaserde::Error::custom(format!("Expected enum tagged with {}, found {:?}", #tag, event)))?
        .value.as_str()
    }
  } else {
    quote! {
      name.local_name.as_str()
    }
  };

  Ok(quote! {
    impl #impl_generics ::yaserde::YaDeserialize for #name #ty_generics #where_clause {
      #[allow(unused_variables)]
      fn deserialize<R: ::std::io::Read>(
        reader: &mut ::yaserde::de::Deserializer<R>,
      ) -> ::std::result::Result<Self, ::yaserde::Error> {
        let (named_element, enum_namespace) =
          if let ::yaserde::__xml::reader::XmlEvent::StartElement{ name, .. } = reader.peek()?.to_owned() {
            (name.local_name.to_owned(), name.namespace.clone())
          } else {
            (::std::string::String::from(#root), ::std::option::Option::None)
          };

        let start_depth = reader.depth();
        ::yaserde::__derive_debug!("Enum {} @ {}: start to parse {:?}", stringify!(#name), start_depth, named_element);

        if !reader.is_flattened_root() {
          #namespaces_matching
        }

        #[allow(unused_assignments, unused_mut)]
        let mut enum_value = ::std::option::Option::None;

        loop {
          let event = reader.peek()?.to_owned();
          ::yaserde::__derive_trace!("Enum {} @ {}: matching {:?}", stringify!(#name), start_depth, event);
          match event {
            ::yaserde::__xml::reader::XmlEvent::StartElement { ref name, ref attributes, .. } => {
              match #element_name {
                #match_to_enum
                _named_element => {
                  let _root = reader.next_event();
                }
              }

              if let ::yaserde::__xml::reader::XmlEvent::Characters(content) = reader.peek()?.to_owned() {
                match content.as_str() {
                  #match_to_enum
                  _ => {}
                }
              }
            }
            ::yaserde::__xml::reader::XmlEvent::EndElement { ref name } => {
              if name.local_name == named_element && reader.depth() == start_depth + 1 {
                break;
              }
              let _root = reader.next_event();
            }
            ::yaserde::__xml::reader::XmlEvent::Characters(ref text_content) => {
              let _root = reader.next_event();
            }
            ::yaserde::__xml::reader::XmlEvent::EndDocument => {
              if #flatten {
                break;
              }

              return ::std::result::Result::Err(
                ::yaserde::Error::custom(::std::format!("End of document, missing some content ?")),
              );
            }
            event => {
              return ::std::result::Result::Err(::yaserde::Error::custom(::std::format!("unknown event {:?}", event)))
            }
          }
        }

        ::yaserde::__derive_debug!("Enum {} @ {}: success", stringify!(#name), start_depth);
        ::std::result::Result::Ok(enum_value.unwrap_or(<#name as ::std::default::Default>::default()))
      }
    }
  })
}

fn parse_variant(variant: &syn::Variant, name: &Ident) -> syn::Result<TokenStream> {
  let xml_element_name = YaSerdeAttribute::from(&variant.attrs).xml_element_name(&variant.ident);

  let variant_name = {
    let label = &variant.ident;
    quote! { #name::#label }
  };

  match variant.fields {
    Fields::Unit => Ok(quote! {
      #xml_element_name => {
        enum_value = ::std::option::Option::Some(#variant_name);
        break;
      }
    }),
    Fields::Unnamed(ref fields) => {
      if fields.unnamed.len() > 1 {
        return Err(syn::Error::new_spanned(
          fields,
          "YaSerDe: enum variants with multiple fields are not supported",
        ));
      }

      let field_visitors = build_unnamed_field_visitors(fields)?;
      let call_visitors = build_unnamed_visitor_calls(fields, &variant_name)?;

      Ok(quote! {
        #xml_element_name => {
          #field_visitors
          #call_visitors
        }
      })
    }
    _ => Ok(TokenStream::new()),
  }
}

fn build_unnamed_field_visitors(fields: &syn::FieldsUnnamed) -> syn::Result<TokenStream> {
  fields
    .unnamed
    .iter()
    .map(|field| YaSerdeField::new(field.clone()))
    .enumerate()
    .map(|(idx, field)| {
      let visitor_label = Ident::new(&format!("__Visitor_{}", idx), field.get_span());

      let make_visitor = |visitor: &Ident, field_type: &TokenStream, fn_body: &TokenStream| {
        quote! {
          #[allow(non_snake_case, non_camel_case_types)]
          struct #visitor_label;
          impl<'de> ::yaserde::Visitor<'de> for #visitor_label {
            type Value = #field_type;

            fn #visitor(
              self,
              v: &str,
            ) -> ::std::result::Result<Self::Value, ::yaserde::Error> {
              #fn_body
            }
          }
        }
      };

      let simple_type_visitor = |simple_type: Field| {
        let visitor = simple_type.get_simple_type_visitor();
        let field_type = simple_type.into();

        make_visitor(
          &visitor,
          &field_type,
          &quote! { #field_type::from_str(v).map_err(::yaserde::Error::custom) },
        )
      };

      match field.get_type() {
        Field::FieldStruct { struct_name } => {
          let struct_id: String = struct_name
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();

          Ok(make_visitor(
            &Ident::new("visit_str", Span::call_site()),
            &quote! { #struct_name },
            &quote! {
              let content = "<".to_string()
                + #struct_id
                + ">"
                + &::yaserde::__xml::escape::escape_str_pcdata(v)
                + "</"
                + #struct_id
                + ">";
              let value: ::std::result::Result<#struct_name, ::yaserde::Error> =
                ::yaserde::de::from_str(&content);
              value
            },
          ))
        }
        Field::FieldOption { data_type } | Field::FieldVec { data_type } => match *data_type {
          Field::FieldStruct { .. } => Ok(TokenStream::new()),
          Field::FieldOption { .. } | Field::FieldVec { .. } => Err(field.unsupported(
            "`Option` or `Vec` nested in `Option` or `Vec` is not supported in enum variants",
          )),
          simple_type => Ok(simple_type_visitor(simple_type)),
        },
        simple_type => Ok(simple_type_visitor(simple_type)),
      }
    })
    .collect()
}

fn build_unnamed_visitor_calls(
  fields: &syn::FieldsUnnamed,
  variant_name: &TokenStream,
) -> syn::Result<TokenStream> {
  let calls = fields
    .unnamed
    .iter()
    .map(|field| YaSerdeField::new(field.clone()))
    .enumerate()
    .filter_map(|(idx, field)| {
      let visitor_label = Ident::new(&format!("__Visitor_{}", idx), field.get_span());

      let call_simple_type_visitor = |simple_type: Field, action| {
        let visitor = simple_type.get_simple_type_visitor();
        let field_type: TokenStream = simple_type.into();

        let label_name = format!("field_{}", idx);

        Some(quote! {
          let visitor = #visitor_label{};

          let value = reader.read_inner_value::<#field_type, _>(|reader| {
            if let ::yaserde::__xml::reader::XmlEvent::EndElement { .. } = *reader.peek()? {
              return visitor.#visitor("");
            }

            if let ::yaserde::__xml::reader::XmlEvent::Characters(s) = reader.next_event()? {
              visitor.#visitor(&s)
            } else {
              ::std::result::Result::Err(
                ::yaserde::Error::custom(::std::format!("unable to parse content for {}", #label_name)),
              )
            }
          })?;

          #action
        })
      };

      let call_struct_visitor = |struct_name, action| {
        Some(quote! {
          match <#struct_name as ::yaserde::YaDeserialize>::deserialize(reader) {
            Ok(value) => {
              #action;
              let _root = reader.next_event();
            },
            Err(msg) => {
              return Err(msg);
            },
          }
        })
      };

      let set_val = quote! {
        enum_value = ::std::option::Option::Some(#variant_name(value))
      };
      let set_opt = quote! {
        enum_value = ::std::option::Option::Some(#variant_name(::std::option::Option::Some(value)))
      };
      let set_vec = quote! {
        match enum_value {
          Some(ref mut v) => match v {
            #variant_name(v) => v.push(value),
            _ => {
              return ::std::result::Result::Err(
                ::yaserde::Error::custom("Got sequence of different types"),
              );
            }
          }
          None => {
            enum_value = ::std::option::Option::Some(#variant_name(vec![value]));
          }
        }
      };

      match field.get_type() {
        Field::FieldStruct { struct_name } => call_struct_visitor(struct_name, set_val),
        Field::FieldOption { data_type } => match *data_type {
          Field::FieldStruct { struct_name } => call_struct_visitor(struct_name, set_opt),
          simple_type => call_simple_type_visitor(simple_type, set_opt),
        },
        Field::FieldVec { data_type } => match *data_type {
          Field::FieldStruct { struct_name } => call_struct_visitor(struct_name, set_vec),
          simple_type => call_simple_type_visitor(simple_type, set_vec),
        },

        simple_type => call_simple_type_visitor(simple_type, set_val),
      }
    })
    .collect();

  Ok(calls)
}
