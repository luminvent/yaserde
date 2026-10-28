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
) -> syn::Result<TokenStream> {
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
    .map(|field| match field.get_type() {
      Field::FieldStruct { struct_name } => {
        Ok(build_default_value(&field, Some(quote!(#struct_name))))
      }
      Field::FieldOption { data_type } => match *data_type {
        Field::FieldVec {
          data_type: inner_data_type,
        } => match *inner_data_type {
          Field::FieldStruct { ref struct_name } => Ok(build_default_value(
            &field,
            Some(quote!(::std::vec::Vec<#struct_name>)),
          )),
          Field::FieldOption { .. } | Field::FieldVec { .. } => {
            Err(field.unsupported("`Option` or `Vec` nested in `Option<Vec<_>>` is not supported"))
          }
          simple_type => {
            let type_token: TokenStream = simple_type.into();
            Ok(build_default_value(
              &field,
              Some(quote!(::std::vec::Vec<#type_token>)),
            ))
          }
        },
        Field::FieldOption { .. } => {
          Err(field.unsupported("`Option` nested in `Option` is not supported"))
        }
        _ => Ok(build_default_value(&field, None)),
      },
      Field::FieldVec { data_type } => match *data_type {
        Field::FieldStruct { ref struct_name } => Ok(build_default_vec_value(
          &field,
          Some(quote!(::std::vec::Vec<#struct_name>)),
        )),
        Field::FieldOption { .. } | Field::FieldVec { .. } => {
          Err(field.unsupported("`Option` or `Vec` nested in `Vec` is not supported"))
        }
        simple_type => {
          let type_token: TokenStream = simple_type.into();

          Ok(build_default_vec_value(
            &field,
            Some(quote!(::std::vec::Vec<#type_token>)),
          ))
        }
      },
      simple_type => {
        let type_token: TokenStream = simple_type.into();

        Ok(build_default_value(&field, Some(type_token)))
      }
    })
    .collect::<syn::Result<_>>()?;

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
            ) -> ::std::result::Result<Self::Value, ::yaserde::Error> {
              // `v` is already unescaped by the XML reader, escape it again before re-parsing
              let content = format!(
                "{}{}{}",
                #xml_opening,
                ::yaserde::__xml::escape::escape_str_pcdata(v),
                #xml_closing,
              );
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
            ) -> ::std::result::Result<Self::Value, ::yaserde::Error> {
              #field_type::from_str(#map_if_bool).map_err(::yaserde::Error::custom)
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
    .map(|field| -> syn::Result<TokenStream> {
      let value_label = field.get_value_label();
      let label_name = field.renamed_label_without_namespace();

      let namespace = field.prefix_namespace(root_attributes);

      let visit_struct = |struct_name: syn::Path, action: TokenStream| {
        Some(quote! {
          (#namespace, #label_name) => {
            if let Ok(::yaserde::__xml::reader::XmlEvent::StartElement { .. }) = reader.peek() {
              // If substruct's start element found then deserialize substruct
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
        Field::FieldOption { .. } | Field::FieldVec { .. } => {
          Err(field.unsupported("`Option` or `Vec` nested in `Option` or `Vec` is not supported"))
        }
        Field::FieldStruct { struct_name } => Ok(visit_struct(struct_name, action)),
        simple_type => Ok(visit_simple(simple_type, action)),
      };

      let visitor = match field.get_type() {
        Field::FieldStruct { struct_name } => Ok(visit_struct(
          struct_name,
          quote! { = ::std::option::Option::Some(value) },
        )),
        Field::FieldOption { data_type } => match *data_type {
          Field::FieldVec {
            data_type: inner_data_type,
          } => match *inner_data_type {
            Field::FieldStruct { struct_name } => {
              // Handle Option<Vec<Struct>>
              Ok(visit_struct(
                struct_name,
                quote! { .get_or_insert_with(::std::vec::Vec::new).push(value) },
              ))
            }
            Field::FieldOption { .. } | Field::FieldVec { .. } => Err(
              field.unsupported("`Option` or `Vec` nested in `Option<Vec<_>>` is not supported"),
            ),
            simple_type => {
              // Handle Option<Vec<simple_type>>
              Ok(visit_simple(
                simple_type,
                quote! { .get_or_insert_with(::std::vec::Vec::new).push(value) },
              ))
            }
          },
          _ => visit_sub(data_type, quote! { = ::std::option::Option::Some(value) }),
        },
        Field::FieldVec { data_type } => visit_sub(data_type, quote! { .push(value) }),
        simple_type => Ok(visit_simple(
          simple_type,
          quote! { = ::std::option::Option::Some(value) },
        )),
      }?;

      Ok(visitor.unwrap_or_default())
    })
    .collect::<syn::Result<_>>()?;

  let call_flatten_visitors: TokenStream = data_struct
    .fields
    .iter()
    .map(|field| YaSerdeField::new(field.clone()))
    .filter(|field| !field.is_attribute() && field.is_flatten())
    .map(|field| {
      let value_label = field.get_value_label();

      match field.get_type() {
        Field::FieldStruct { .. } => Ok(quote! {
          #value_label = Some(::yaserde::de::from_str_flattened(&unused_xml_elements)?);
        }),
        Field::FieldOption { data_type } => match *data_type {
          Field::FieldStruct { .. } => Ok(quote! {
            #value_label = ::yaserde::de::from_str_flattened(&unused_xml_elements).ok();
          }),
          _ => Err(field.unsupported("`flatten` is only supported on struct fields")),
        },
        _ => Err(field.unsupported("`flatten` is only supported on struct fields")),
      }
    })
    .collect::<syn::Result<_>>()?;

  let attributes_loading: TokenStream = data_struct
    .fields
    .iter()
    .map(|field| YaSerdeField::new(field.clone()))
    .filter(|field| field.is_attribute())
    .map(|field| -> syn::Result<TokenStream> {
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
              let values = #label.get_or_insert_with(::std::vec::Vec::new);
              for value in attr.value.split_whitespace() {
                let visitor = #visitor_label{};
                let value = visitor.#visitor(value)?;
                values.push(value);
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

      let nested_error = || {
        Err(field.unsupported(
          "`Option` or `Vec` nested in `Option` or `Vec` is not supported for attributes",
        ))
      };

      let visit_sub = |sub_type: Box<Field>, action: TokenStream| match *sub_type {
        Field::FieldOption { .. } => nested_error(),
        Field::FieldVec { data_type } => match data_type.as_ref() {
          Field::FieldStruct { struct_name } => Ok(visit_option_vec(
            &Ident::new("visit_str", field.get_span()),
            &field.get_visitor_ident(Some(struct_name)),
          )),
          Field::FieldOption { .. } | Field::FieldVec { .. } => nested_error(),
          simple_type => Ok(visit_option_vec(
            &simple_type.get_simple_type_visitor(),
            &field.get_visitor_ident(None),
          )),
        },
        Field::FieldStruct { struct_name } => Ok(visit_struct(struct_name, action)),
        simple_type => Ok(visit_simple(simple_type, action)),
      };

      let visitor = match field.get_type() {
        Field::FieldString => Ok(visit_string()),
        Field::FieldOption { data_type } => {
          visit_sub(data_type, quote! { = ::std::option::Option::Some(value) })
        }
        Field::FieldVec { data_type } => match data_type.as_ref() {
          Field::FieldStruct { struct_name } => Ok(visit_vec(
            &quote! { .push(value) },
            &Ident::new("visit_str", field.get_span()),
            &field.get_visitor_ident(Some(struct_name)),
          )),
          Field::FieldOption { .. } | Field::FieldVec { .. } => nested_error(),
          simple_type => Ok(visit_vec(
            &quote! { .push(value) },
            &simple_type.get_simple_type_visitor(),
            &visitor_label,
          )),
        },
        Field::FieldStruct { struct_name } => {
          Ok(visit_struct(struct_name, quote! { = Some(value) }))
        }
        simple_type => Ok(visit_simple(simple_type, quote! { = Some(value) })),
      }?;

      Ok(visitor.unwrap_or_default())
    })
    .collect::<syn::Result<_>>()?;

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
          set_text(&quote! {
            ::std::option::Option::Some(#type_token::from_str(text_content).map_err(::yaserde::Error::custom)?)
          })
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

            quote! { #label: #value_label.ok_or_else(|| ::yaserde::Error::custom(#error))?, }
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

  let flatten = root_attributes.flatten;
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

  Ok(quote! {
    impl #impl_generics ::yaserde::YaDeserialize for #name #ty_generics #where_clause {
      #[allow(unused_variables)]
      fn deserialize<R: ::std::io::Read>(
        reader: &mut ::yaserde::de::Deserializer<R>,
      ) -> ::std::result::Result<Self, ::yaserde::Error> {
        let (named_element, struct_namespace) =
          if let ::yaserde::__xml::reader::XmlEvent::StartElement { name, .. } = reader.peek()?.to_owned() {
            (name.local_name.to_owned(), name.namespace.clone())
          } else {
            (::std::string::String::from(#root), ::std::option::Option::None)
          };
        let start_depth = reader.depth();
        ::yaserde::__derive_debug!("Struct {} @ {}: start to parse {:?}", stringify!(#name), start_depth,
               named_element);

        if reader.depth() == 0 && !reader.is_flattened_root() {
          #namespaces_matching
        }

        #variables
        #field_visitors
        #init_unused

        // The first start element is the struct's own element. A flattened struct has none: its first start
        // element is either a child, or the parent's element when the struct is flattened into another one.
        let mut first_element = true;

        loop {
          let event = reader.peek()?.to_owned();
          ::yaserde::__derive_trace!(
            "Struct {} @ {}: matching {:?}",
            stringify!(#name), start_depth, event,
          );
          match event {
            ::yaserde::__xml::reader::XmlEvent::StartElement{ref name, ref attributes, ..} => {
              let namespace = name.namespace.clone().unwrap_or_default();
              if first_element && !#flatten {
                // Consume the struct's own element first: when it shares a name with a child element, it must
                // not be matched as that child below. It is not compared with the struct's name, as the parent
                // field may rename it.
                let event = reader.next_event()?;
                #write_unused
              } else {
                match (namespace.as_str(), name.local_name.as_str()) {
                  #call_visitors
                  _ => {
                    let event = reader.next_event()?;
                    #write_unused

                    // The parent's element of a flattened struct is entered, not skipped
                    if !first_element {
                      reader.skip_element(|event| {
                        #write_unused
                      })?;
                    }
                  }
                }
              }
              if first_element {
                // Attributes are only read from the struct's own element
                #attributes_loading
                first_element = false;
              }
            }
            ::yaserde::__xml::reader::XmlEvent::EndElement { ref name } => {
              if name.local_name == named_element && reader.depth() == start_depth + 1 {
                #write_unused
                break;
              }
              let event = reader.next_event()?;
              #write_unused
            }
            ::yaserde::__xml::reader::XmlEvent::EndDocument => {
              if #flatten {
                break;
              }
            }
            ::yaserde::__xml::reader::XmlEvent::Characters(ref text_content) => {
              #set_text
              let event = reader.next_event()?;
              #write_unused
            }
            event => {
              return ::std::result::Result::Err(::yaserde::Error::custom(::std::format!("unknown event {:?}", event)));
            }
          }
        }

        #visit_unused

        ::yaserde::__derive_debug!("Struct {} @ {}: success", stringify!(#name), start_depth);
        ::std::result::Result::Ok(#name{#struct_builder})
      }
    }
  })
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

      let value = reader.read_inner_value::<#field_type, _>(|reader| {
        match reader.peek()? {
          ::yaserde::__xml::reader::XmlEvent::Characters(s) => {
            let val = visitor.#visitor(&s);
            let _event = reader.next_event()?;
            val
          }
          // Empty element, e.g. `<field></field>` or `<field/>`
          ::yaserde::__xml::reader::XmlEvent::EndElement { .. } => visitor.#visitor(""),
          _ => ::std::result::Result::Err(::yaserde::Error::custom(::std::format!("unable to parse content for {}", #label_name))),
        }
      }).map_err(|e| e.in_field(#label_name))?;

      #value_label #action
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
