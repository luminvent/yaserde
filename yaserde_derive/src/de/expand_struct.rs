use super::build_default_value::{build_default_value, build_default_vec_value};
use crate::common::{Field, YaSerdeAttribute, YaSerdeField};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DataStruct, Generics, Ident};

pub fn parse(
  data_struct: &DataStruct,
  name: &Ident,
  root: &str,
  root_attributes: &YaSerdeAttribute,
  generics: &Generics,
) -> TokenStream {
  let namespaces_matching = root_attributes.get_namespace_matching(
    &None,
    quote!(struct_namespace),
    quote!(named_element),
    true,
  );

  let variables: TokenStream = data_struct
    .fields
    .iter()
    .map(|field| YaSerdeField::new(field.clone()))
    .filter_map(|field| match field.get_type() {
      Field::FieldStruct { struct_name } => build_default_value(&field, Some(quote!(#struct_name))),
      Field::FieldOption { data_type } => match *data_type {
        Field::FieldVec {
          data_type: inner_data_type,
        } => match *inner_data_type {
          Field::FieldStruct { ref struct_name } => {
            build_default_value(&field, Some(quote!(::std::vec::Vec<#struct_name>)))
          }
          simple_type => {
            let type_token: TokenStream = simple_type.into();
            build_default_value(&field, Some(quote!(::std::vec::Vec<#type_token>)))
          }
        },
        _ => build_default_value(&field, None),
      },
      Field::FieldVec { data_type } => match *data_type {
        Field::FieldStruct { ref struct_name } => {
          build_default_vec_value(&field, Some(quote!(::std::vec::Vec<#struct_name>)))
        }
        Field::FieldOption { .. } | Field::FieldVec { .. } => {
          unimplemented!("Option or Vec nested in Vec<>");
        }
        simple_type => {
          let type_token: TokenStream = simple_type.into();

          build_default_vec_value(&field, Some(quote!(::std::vec::Vec<#type_token>)))
        }
      },
      simple_type => {
        let type_token: TokenStream = simple_type.into();

        build_default_value(&field, Some(type_token))
      }
    })
    .collect();

  let field_visitors: TokenStream = data_struct
    .fields
    .iter()
    .map(|field| YaSerdeField::new(field.clone()))
    .filter(|field| {
      if field.is_attribute() {
        return true;
      };
      match field.get_type() {
        Field::FieldVec { data_type } => !matches!(*data_type, Field::FieldStruct { .. }),
        Field::FieldOption { data_type } => !matches!(*data_type, Field::FieldStruct { .. }),
        Field::FieldStruct { .. } => false,
        _ => true,
      }
    })
    .filter_map(|field| {
      let struct_visitor = |struct_name: syn::Path| {
        let struct_id: String = struct_name
          .segments
          .iter()
          .map(|s| s.ident.to_string())
          .collect();

        let visitor_label = field.get_visitor_ident(Some(&struct_name));

        let xml_opening = format!("<{struct_id}>");
        let xml_closing = format!("</{struct_id}>");

        Some(quote! {
          #[allow(non_snake_case, non_camel_case_types)]
          struct #visitor_label;
          impl<'de> ::yaserde::Visitor<'de> for #visitor_label {
            type Value = #struct_name;

            fn visit_str(
              self,
              v: &str,
            ) -> ::std::result::Result<Self::Value, ::std::string::String> {
              let content = format!("{}{}{}", #xml_opening, v, #xml_closing);
              ::yaserde::de::from_str(&content)
            }
          }
        })
      };

      let simple_type_visitor = |simple_type: Field| {
        let visitor = simple_type.get_simple_type_visitor();
        let visitor_label = field.get_visitor_ident(None);
        let field_type = TokenStream::from(simple_type);

        let map_if_bool = if field_type.to_string() == "bool" {
          quote!(match v {
            "1" => "true",
            "0" => "false",
            _ => v,
          })
        } else {
          quote!(v)
        };

        Some(quote! {
          #[allow(non_snake_case, non_camel_case_types)]
          struct #visitor_label;
          impl<'de> ::yaserde::Visitor<'de> for #visitor_label {
            type Value = #field_type;

            fn #visitor(
              self,
              v: &str,
            ) -> ::std::result::Result<Self::Value, ::std::string::String> {
              #field_type::from_str(#map_if_bool).map_err(|e| e.to_string())
            }
          }
        })
      };

      match field.get_type() {
        Field::FieldStruct { struct_name } => struct_visitor(struct_name),
        Field::FieldOption { data_type } => match *data_type {
          Field::FieldStruct { struct_name } => struct_visitor(struct_name),
          Field::FieldOption { .. } => None,
          Field::FieldVec { data_type } => match *data_type {
            Field::FieldStruct { struct_name } => struct_visitor(struct_name),
            Field::FieldOption { .. } | Field::FieldVec { .. } => None,
            simple_type => simple_type_visitor(simple_type),
          },
          simple_type => simple_type_visitor(simple_type),
        },
        Field::FieldVec { data_type } => match *data_type {
          Field::FieldStruct { struct_name } => struct_visitor(struct_name),
          Field::FieldOption { .. } | Field::FieldVec { .. } => None,
          simple_type => simple_type_visitor(simple_type),
        },
        simple_type => simple_type_visitor(simple_type),
      }
    })
    .collect();

  let call_visitors: TokenStream = data_struct
    .fields
    .iter()
    .map(|field| YaSerdeField::new(field.clone()))
    .filter(|field| !field.is_attribute() && !field.is_flatten())
    .filter_map(|field| {
      let value_label = field.get_value_label();
      let label_name = field.renamed_label_without_namespace();

      let namespace = field.prefix_namespace(root_attributes);

      let visit_struct = |struct_name: syn::Path, action: TokenStream| {
        Some(quote! {
          (#namespace, #label_name) => {
            if depth == 0 {
              // Don't count current struct's StartElement as substruct's StartElement
              let _root = reader.next_event();
            }
            if let Ok(::yaserde::__xml::reader::XmlEvent::StartElement { .. }) = reader.peek() {
              // If substruct's start element found then deserialize substruct
              reader.inner_struct_label = Some(#label_name);
              let value = <#struct_name as ::yaserde::YaDeserialize>::deserialize(reader)?;
              #value_label #action;
              // read EndElement
              let _event = reader.next_event()?;
            }
          }
        })
      };

      let visit_simple = |simple_type: Field, action: TokenStream| {
        let field_visitor = simple_type.get_simple_type_visitor();
        let field_type = TokenStream::from(simple_type);
        build_call_visitor(
          &field_type,
          &field_visitor,
          &action,
          &field,
          root_attributes,
        )
      };

      let visit_sub = |sub_type: Box<Field>, action: TokenStream| match *sub_type {
        Field::FieldOption { .. } | Field::FieldVec { .. } => unimplemented!(),
        Field::FieldStruct { struct_name } => visit_struct(struct_name, action),
        simple_type => visit_simple(simple_type, action),
      };

      match field.get_type() {
        Field::FieldStruct { struct_name } => {
          visit_struct(struct_name, quote! { = ::std::option::Option::Some(value) })
        }
        Field::FieldOption { data_type } => match *data_type {
          Field::FieldVec {
            data_type: inner_data_type,
          } => match *inner_data_type {
            Field::FieldStruct { struct_name } => {
              // Handle Option<Vec<Struct>>
              visit_struct(
                struct_name,
                quote! {
                  = if #value_label.is_some() {
                    #value_label.as_mut().unwrap().push(value);
                    #value_label
                  } else {
                    Some(vec![value])
                  }
                },
              )
            }
            simple_type => {
              // Handle Option<Vec<simple_type>>
              visit_simple(
                simple_type,
                quote! {
                  = if #value_label.is_some() {
                    #value_label.as_mut().unwrap().push(value);
                    #value_label
                  } else {
                    Some(vec![value])
                  }
                },
              )
            }
          },
          _ => visit_sub(data_type, quote! { = ::std::option::Option::Some(value) }),
        },
        Field::FieldVec { data_type } => visit_sub(data_type, quote! { .push(value) }),
        simple_type => visit_simple(simple_type, quote! { = ::std::option::Option::Some(value) }),
      }
    })
    .collect();

  let call_flatten_visitors: TokenStream = data_struct
    .fields
    .iter()
    .map(|field| YaSerdeField::new(field.clone()))
    .filter(|field| !field.is_attribute() && field.is_flatten())
    .map(|field| {
      let value_label = field.get_value_label();

      match field.get_type() {
        Field::FieldStruct { .. } => quote! {
          #value_label = Some(::yaserde::de::from_str(&unused_xml_elements)?);
        },
        Field::FieldOption { data_type } => match *data_type {
          Field::FieldStruct { .. } => quote! {
            #value_label = ::yaserde::de::from_str(&unused_xml_elements).ok();
          },
          field_type => unimplemented!(r#""flatten" is not implemented for {:?}"#, field_type),
        },
        field_type => unimplemented!(r#""flatten" is not implemented for {:?}"#, field_type),
      }
    })
    .collect();

  let attributes_loading: TokenStream = data_struct
    .fields
    .iter()
    .map(|field| YaSerdeField::new(field.clone()))
    .filter(|field| field.is_attribute())
    .filter_map(|field| {
      let label = field.get_value_label();
      let label_name = field.renamed_label_without_namespace();
      let visitor_label = field.get_visitor_ident(None);

      let visit = |action: &TokenStream, visitor: &Ident, visitor_label: &Ident| {
        Some(quote! {
          for attr in attributes {
            if attr.name.local_name == #label_name {
              let visitor = #visitor_label{};
              let value = visitor.#visitor(&attr.value)?;
              #label #action;
            }
          }
        })
      };

      let visit_vec = |action: &TokenStream, visitor: &Ident, visitor_label: &Ident| {
        Some(quote! {
          for attr in attributes {
            if attr.name.local_name == #label_name {
              for value in attr.value.split_whitespace() {
                let visitor = #visitor_label{};
                let value = visitor.#visitor(value)?;
                #label #action;
              }
            }
          }
        })
      };

      let visit_option_vec = |visitor: &Ident, visitor_label: &Ident| {
        Some(quote! {
          for attr in attributes {
            if attr.name.local_name == #label_name {
              if #label.is_none() {
                #label = Some(Vec::new());
              }
              for value in attr.value.split_whitespace() {
                let visitor = #visitor_label{};
                let value = visitor.#visitor(value)?;
                #label.as_mut().unwrap().push(value);
              }
            }
          }
        })
      };

      let visit_string = || {
        Some(quote! {
          for attr in attributes {
            if attr.name.local_name == #label_name {
              #label = Some(attr.value.to_owned());
            }
          }
        })
      };

      let visit_struct = |struct_name: syn::Path, action: TokenStream| {
        visit(
          &action,
          &Ident::new("visit_str", Span::call_site()),
          &field.get_visitor_ident(Some(&struct_name)),
        )
      };

      let visit_simple = |simple_type: Field, action: TokenStream| {
        visit(
          &action,
          &simple_type.get_simple_type_visitor(),
          &visitor_label,
        )
      };

      let visit_sub = |sub_type: Box<Field>, action: TokenStream| match *sub_type {
        Field::FieldOption { .. } => unimplemented!(),
        Field::FieldVec { data_type } => match data_type.as_ref() {
          Field::FieldStruct { struct_name } => visit_option_vec(
            &Ident::new("visit_str", field.get_span()),
            &field.get_visitor_ident(Some(struct_name)),
          ),
          Field::FieldOption { .. } | Field::FieldVec { .. } => unimplemented!("Not supported"),
          simple_type => visit_option_vec(
            &simple_type.get_simple_type_visitor(),
            &field.get_visitor_ident(None),
          ),
        },
        Field::FieldStruct { struct_name } => visit_struct(struct_name, action),
        simple_type => visit_simple(simple_type, action),
      };

      match field.get_type() {
        Field::FieldString => visit_string(),
        Field::FieldOption { data_type } => {
          visit_sub(data_type, quote! { = ::std::option::Option::Some(value) })
        }
        Field::FieldVec { data_type } => match data_type.as_ref() {
          Field::FieldStruct { struct_name } => visit_vec(
            &quote! { .push(value) },
            &Ident::new("visit_str", field.get_span()),
            &field.get_visitor_ident(Some(struct_name)),
          ),
          Field::FieldOption { .. } | Field::FieldVec { .. } => unimplemented!("Not supported"),
          simple_type => visit_vec(
            &quote! { .push(value) },
            &simple_type.get_simple_type_visitor(),
            &visitor_label,
          ),
        },
        Field::FieldStruct { struct_name } => visit_struct(struct_name, quote! { = Some(value) }),
        simple_type => visit_simple(simple_type, quote! { = Some(value) }),
      }
    })
    .collect();

  let set_text: TokenStream = data_struct
    .fields
    .iter()
    .map(|field| YaSerdeField::new(field.clone()))
    .filter_map(|field| {
      let label = field.get_value_label();

      let set_text = |action: &TokenStream| {
        field
          .is_text_content()
          .then_some(quote! { #label = #action; })
      };

      match field.get_type() {
        Field::FieldString => set_text(&quote! { Some(text_content.to_owned()) }),
        Field::FieldOption { data_type } => match *data_type {
          Field::FieldString => set_text(
            &quote! { if text_content.is_empty() { None } else { Some(text_content.to_owned()) }},
          ),
          _ => None,
        },
        Field::FieldStruct { .. } | Field::FieldVec { .. } => None,
        simple_type => {
          let type_token = TokenStream::from(simple_type);
          set_text(&quote! { #type_token::from_str(text_content).unwrap() })
        }
      }
    })
    .collect();

  let struct_builder: TokenStream = data_struct
    .fields
    .iter()
    .map(|field| YaSerdeField::new(field.clone()))
    .map(|field| {
      let label = &field.label();
      let value_label = field.get_value_label();

      match field.get_type() {
        Field::FieldOption { .. } | Field::FieldVec { .. } => {
          quote! { #label: #value_label, }
        }
        _ => {
          if let Some(default_function) = field.get_default_function() {
            quote! { #label: #value_label.unwrap_or_else(|| #default_function()), }
          } else {
            let error = format!(
              "{} is a required field of {}",
              label
                .as_ref()
                .map(|label| label.to_string())
                .unwrap_or_default(),
              name
            );

            quote! { #label: #value_label.ok_or_else(|| #error.to_string())?, }
          }
        }
      }
    })
    .collect();

  let (init_unused, write_unused, visit_unused) = if call_flatten_visitors.is_empty() {
    (None, None, None)
  } else {
    build_code_for_unused_xml_events(&call_flatten_visitors)
  };

  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

  quote! {
    impl #impl_generics ::yaserde::YaDeserialize for #name #ty_generics #where_clause {
      #[allow(unused_variables)]
      fn deserialize<R: ::std::io::Read>(
        reader: &mut ::yaserde::de::Deserializer<R>,
      ) -> ::std::result::Result<Self, ::std::string::String> {
        let root_label= reader.inner_struct_label.take().unwrap_or(#root);
        let (named_element, struct_namespace) =
          if let ::yaserde::__xml::reader::XmlEvent::StartElement { name, .. } = reader.peek()?.to_owned() {
            (name.local_name.to_owned(), name.namespace.clone())
          } else {
            (::std::string::String::from(#root), ::std::option::Option::None)
          };
        let start_depth = reader.depth();
        ::yaserde::__derive_debug!("Struct {} @ {}: start to parse {:?}", stringify!(#name), start_depth,
               named_element);

        if reader.depth() == 0 {
          #namespaces_matching
        }

        #variables
        #field_visitors
        #init_unused

        let mut depth = 0;

        loop {
          let event = reader.peek()?.to_owned();
          ::yaserde::__derive_trace!(
            "Struct {} @ {}: matching {:?}",
            stringify!(#name), start_depth, event,
          );
          match event {
            ::yaserde::__xml::reader::XmlEvent::StartElement{ref name, ref attributes, ..} => {
              let namespace = name.namespace.clone().unwrap_or_default();
              if depth == 0 && name.local_name == root_label {
                // Consume root element. We must do this first. In the case it shares a name with a child element, we don't
                // want to prematurely match the child element below.
                let event = reader.next_event()?;
                #write_unused
              } else {

                match (namespace.as_str(), name.local_name.as_str()) {
                  #call_visitors
                  _ => {
                    let event = reader.next_event()?;
                    #write_unused

                    if depth > 0 { // Don't skip root element
                      reader.skip_element(|event| {
                        #write_unused
                      })?;
                    }
                  }
                }
              }
              if depth == 0 { // Look for attributes only at element start
                #attributes_loading
              }
              depth += 1;
            }
            ::yaserde::__xml::reader::XmlEvent::EndElement { ref name } => {
              if name.local_name == named_element && reader.depth() == start_depth + 1 {
                #write_unused
                break;
              }
              let event = reader.next_event()?;
              #write_unused
              depth -= 1;
            }
            ::yaserde::__xml::reader::XmlEvent::EndDocument => {
              // once we receive this once, we will keep getting it, potentially looping forever
              break;
            }
            ::yaserde::__xml::reader::XmlEvent::Characters(ref text_content) => {
              #set_text
              let event = reader.next_event()?;
              #write_unused
            }
            event => {
              return ::std::result::Result::Err(::std::format!("unknown event {:?}", event));
            }
          }
        }

        #visit_unused

        ::yaserde::__derive_debug!("Struct {} @ {}: success", stringify!(#name), start_depth);
        ::std::result::Result::Ok(#name{#struct_builder})
      }
    }
  }
}

fn build_call_visitor(
  field_type: &TokenStream,
  visitor: &Ident,
  action: &TokenStream,
  field: &YaSerdeField,
  root_attributes: &YaSerdeAttribute,
) -> Option<TokenStream> {
  let value_label = field.get_value_label();
  let label_name = field.renamed_label_without_namespace();
  let visitor_label = field.get_visitor_ident(None);

  let namespaces_matching = field.get_namespace_matching(
    root_attributes,
    quote!(name.namespace.as_ref()),
    quote!(name.local_name.as_str()),
  );

  let namespace = field.prefix_namespace(root_attributes);

  Some(quote! {
    (#namespace, #label_name) => {
      let visitor = #visitor_label{};

      #namespaces_matching

      let result = reader.read_inner_value::<#field_type, _>(|reader| {
        if let ::std::result::Result::Ok(::yaserde::__xml::reader::XmlEvent::Characters(s)) = reader.peek() {
          let val = visitor.#visitor(&s);
          let _event = reader.next_event()?;
          val
        } else {
          ::std::result::Result::Err(::std::format!("unable to parse content for {}", #label_name))
        }
      });

      if let ::std::result::Result::Ok(value) = result {
        #value_label#action
      }
    }
  })
}

fn build_code_for_unused_xml_events(
  call_flatten_visitors: &TokenStream,
) -> (
  Option<TokenStream>,
  Option<TokenStream>,
  Option<TokenStream>,
) {
  (
    Some(quote! {
      let mut buf = ::std::vec![];
      let mut writer = ::std::option::Option::Some(::yaserde::__xml::writer::EventWriter::new(&mut buf));
    }),
    Some(quote! {
      if let ::std::option::Option::Some(ref mut w) = writer {
        if w.write(event.as_writer_event().unwrap()).is_err() {
          writer = ::std::option::Option::None;
        }
      }
    }),
    Some(quote! {
      if writer.is_some() {
        let unused_xml_elements = ::std::string::String::from_utf8(buf).unwrap();
        #call_flatten_visitors
      }
    }),
  )
}
