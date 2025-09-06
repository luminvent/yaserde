use crate::common::{Field, YaSerdeAttribute, YaSerdeField};

use crate::ser::{element::*, implement_serializer::implement_serializer};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;
use syn::{DataStruct, Generics};

pub fn serialize(
  data_struct: &DataStruct,
  name: &Ident,
  root: &str,
  root_attributes: &YaSerdeAttribute,
  generics: &Generics,
) -> TokenStream {
  let append_attributes: TokenStream = data_struct
    .fields
    .iter()
    .map(|field| YaSerdeField::new(field.clone()))
    .filter(|field| field.is_attribute() || field.is_flatten())
    .map(|field| {
      let label = field.label();

      if field.is_attribute() {
        let label_name = field.renamed_label(root_attributes);

        match field.get_type() {
          Field::FieldString
          | Field::FieldBool
          | Field::FieldI8
          | Field::FieldU8
          | Field::FieldI16
          | Field::FieldU16
          | Field::FieldI32
          | Field::FieldU32
          | Field::FieldI64
          | Field::FieldU64
          | Field::FieldF32
          | Field::FieldF64 => field.ser_wrap_default_attribute(
            Some(quote!(self.#label.to_string())),
            quote!({
              struct_start_event.attr(#label_name, &yaserde_inner)
            }),
          ),
          Field::FieldOption { data_type } => match *data_type {
            Field::FieldString => field.ser_wrap_default_attribute(
              None,
              quote!({
                if let ::std::option::Option::Some(ref value) = self.#label {
                  struct_start_event.attr(#label_name, value)
                } else {
                  struct_start_event
                }
              }),
            ),
            Field::FieldBool
            | Field::FieldI8
            | Field::FieldU8
            | Field::FieldI16
            | Field::FieldU16
            | Field::FieldI32
            | Field::FieldU32
            | Field::FieldI64
            | Field::FieldU64
            | Field::FieldF32
            | Field::FieldF64 => field.ser_wrap_default_attribute(
              Some(
                quote!(self.#label.map_or_else(|| ::std::string::String::new(), |v| v.to_string())),
              ),
              quote!({
                if let ::std::option::Option::Some(ref value) = self.#label {
                  struct_start_event.attr(#label_name, &yaserde_inner)
                } else {
                  struct_start_event
                }
              }),
            ),
            Field::FieldVec { .. } => {
              let item_ident = Ident::new("yaserde_item", field.get_span());
              let inner = enclose_formatted_characters(&item_ident, label_name);

              field.ser_wrap_default_attribute(
                None,
                quote!({
                  if let ::std::option::Option::Some(ref yaserde_list) = self.#label {
                    for yaserde_item in yaserde_list.iter() {
                      #inner
                    }
                  }
                }),
              )
            }
            Field::FieldStruct { .. } => field.ser_wrap_default_attribute(
              Some(quote! {
              self.#label
                .as_ref()
                .map_or_else(
                  || ::std::result::Result::Ok(::std::string::String::new()),
                  |v| ::yaserde::ser::to_string_content(v),
                )?
              }),
              quote!({
                if let ::std::option::Option::Some(ref yaserde_struct) = self.#label {
                  struct_start_event.attr(#label_name, &yaserde_inner)
                } else {
                  struct_start_event
                }
              }),
            ),
            Field::FieldOption { .. } => unimplemented!(),
          },
          Field::FieldStruct { .. } => field.ser_wrap_default_attribute(
            Some(quote! { ::yaserde::ser::to_string_content(&self.#label)? }),
            quote!({
              struct_start_event.attr(#label_name, &yaserde_inner)
            }),
          ),
          Field::FieldVec { data_type } => match *data_type {
            Field::FieldString
            | Field::FieldBool
            | Field::FieldI8
            | Field::FieldU8
            | Field::FieldI16
            | Field::FieldU16
            | Field::FieldI32
            | Field::FieldU32
            | Field::FieldI64
            | Field::FieldU64
            | Field::FieldF32
            | Field::FieldF64 => field.ser_wrap_default_attribute(
              Some(quote! {
                self.#label
                  .iter()
                  .map(|item| item.to_string())
                  .collect::<::std::vec::Vec<_>>()
                  .join(" ")
              }),
              quote!({
                struct_start_event.attr(#label_name, &yaserde_inner)
              }),
            ),
            Field::FieldOption { .. } | Field::FieldVec { .. } => {
              unimplemented!("Nested Option or Vec in Vec not supported for attributes")
            }
            Field::FieldStruct { .. } => field.ser_wrap_default_attribute(
              Some(quote! {
                self.#label
                  .iter()
                  .map(|item| ::yaserde::ser::to_string_content(item))
                  .collect::<::std::result::Result<::std::vec::Vec<_>, _>>()?
                  .join(" ")
              }),
              quote!({
                struct_start_event.attr(#label_name, &yaserde_inner)
              }),
            ),
          },
        }
      } else {
        match field.get_type() {
          Field::FieldStruct { .. } => {
            quote!(
              let (attributes, namespace) = self.#label.serialize_attributes(
                ::std::vec![],
                ::yaserde::__xml::namespace::Namespace::empty(),
              )?;
              child_attributes_namespace.extend(&namespace);
              child_attributes.extend(attributes);
            )
          }
          _ => quote!(),
        }
      }
    })
    .collect();

  let struct_inspector: TokenStream = data_struct
    .fields
    .iter()
    .map(|field| YaSerdeField::new(field.clone()))
    .filter(|field| !field.is_attribute())
    .filter_map(|field| {
      let label = field.label();
      if field.is_text_content() {
        return match field.get_type() {
          Field::FieldOption { .. } => Some(quote!(
            let s = self.#label.as_deref().unwrap_or_default();
            let data_event = ::yaserde::__xml::writer::XmlEvent::characters(s);
            writer.write(data_event).map_err(|e| e.to_string())?;
          )),
          _ => Some(quote!(
            let data_event = ::yaserde::__xml::writer::XmlEvent::characters(&self.#label);
            writer.write(data_event).map_err(|e| e.to_string())?;
          )),
        };
      }
      let label_name = field.renamed_label(root_attributes);
      let conditions = condition_generator(&label, &field);

      if field.is_cdata() {
        return quote! {
            #conditions {
              let start_event = ::yaserde::__xml::writer::XmlEvent::start_element(#label_name);
              writer.write(start_event).map_err(|e| e.to_string())?;
              let data = ::yaserde::__xml::writer::events::XmlEvent::cdata(&self.#label);
              writer.write(data).map_err(|e| e.to_string())?;
              let end_event = ::yaserde::__xml::writer::XmlEvent::end_element();
              writer.write(end_event).map_err(|e| e.to_string())?;
            }
        }.into()
      }

      match field.get_type() {
        Field::FieldString
        | Field::FieldBool
        | Field::FieldI8
        | Field::FieldU8
        | Field::FieldI16
        | Field::FieldU16
        | Field::FieldI32
        | Field::FieldU32
        | Field::FieldI64
        | Field::FieldU64
        | Field::FieldF32
        | Field::FieldF64 => serialize_element(&label, label_name, &conditions),

        Field::FieldOption { data_type } => match *data_type {
          Field::FieldString
          | Field::FieldBool
          | Field::FieldI8
          | Field::FieldU8
          | Field::FieldI16
          | Field::FieldU16
          | Field::FieldI32
          | Field::FieldU32
          | Field::FieldI64
          | Field::FieldU64
          | Field::FieldF32
          | Field::FieldF64 => {
            let item_ident = Ident::new("yaserde_item", field.get_span());
            let inner = enclose_formatted_characters_for_value(&item_ident, label_name);

            Some(quote! {
              #conditions {
                if let Some(ref yaserde_item) = self.#label {
                  #inner
                }
              }
            })
          }
          Field::FieldVec { .. } => {
            let item_ident = Ident::new("yaserde_item", field.get_span());
            let inner = enclose_formatted_characters_for_value(&item_ident, label_name);

            Some(quote! {
              #conditions {
                if let ::std::option::Option::Some(ref yaserde_items) = &self.#label {
                  for yaserde_item in yaserde_items.iter() {
                    #inner
                  }
                }
              }
            })
          }
          Field::FieldStruct { .. } => Some(if field.is_flatten() {
            quote! {
              #conditions {
                if let ::std::option::Option::Some(ref item) = &self.#label {
                  writer.set_start_event_name(::std::option::Option::None);
                  writer.set_skip_start_end(true);
                  ::yaserde::YaSerialize::serialize(item, writer)?;
                }
              }
            }
          } else {
            quote! {
              #conditions {
                if let ::std::option::Option::Some(ref item) = &self.#label {
                  writer.set_start_event_name(::std::option::Option::Some(#label_name.to_string()));
                  writer.set_skip_start_end(false);
                  ::yaserde::YaSerialize::serialize(item, writer)?;
                }
              }
            }
          }),
          _ => unimplemented!(),
        },
        Field::FieldStruct { .. } => {
          let (start_event, skip_start) = if field.is_flatten() {
            (quote!(::std::option::Option::None), true)
          } else {
            (
              quote!(::std::option::Option::Some(#label_name.to_string())),
              false,
            )
          };

          Some(quote! {
            #conditions {
              writer.set_start_event_name(#start_event);
              writer.set_skip_start_end(#skip_start);
              ::yaserde::YaSerialize::serialize(&self.#label, writer)?;
            }
          })
        }
        Field::FieldVec { data_type } => match *data_type {
          Field::FieldString => {
            let item_ident = Ident::new("yaserde_item", field.get_span());
            let inner = enclose_formatted_characters_for_value(&item_ident, label_name);

            Some(quote! {
              #conditions {
                for yaserde_item in &self.#label {
                  #inner
                }
              }
            })
          }
          Field::FieldBool
          | Field::FieldI8
          | Field::FieldU8
          | Field::FieldI16
          | Field::FieldU16
          | Field::FieldI32
          | Field::FieldU32
          | Field::FieldI64
          | Field::FieldU64
          | Field::FieldF32
          | Field::FieldF64 => {
            let item_ident = Ident::new("yaserde_item", field.get_span());
            let inner = enclose_formatted_characters_for_value(&item_ident, label_name);

            Some(quote! {
              #conditions {
                for yaserde_item in &self.#label {
                  #inner
                }
              }
            })
          }
          Field::FieldOption { .. } => Some(quote! {
            #conditions {
              for item in &self.#label {
                if let Some(value) = item {
                  writer.set_start_event_name(None);
                  writer.set_skip_start_end(false);
                  ::yaserde::YaSerialize::serialize(value, writer)?;
                }
              }
            }
          }),
          Field::FieldStruct { .. } => {
            if field.is_flatten() {
              Some(quote! {
                #conditions {
                  for item in &self.#label {
                      writer.set_start_event_name(::std::option::Option::None);
                    writer.set_skip_start_end(true);
                    ::yaserde::YaSerialize::serialize(item, writer)?;
                  }
                }
              })
            } else {
              Some(quote! {
                #conditions {
                  for item in &self.#label {
                    writer.set_start_event_name(::std::option::Option::Some(#label_name.to_string()));
                    writer.set_skip_start_end(false);
                    ::yaserde::YaSerialize::serialize(item, writer)?;
                 }
                }
              })
            }
            /*let (start_event, skip_start) = if field.is_flatten() {
              (quote!(None), true)
            } else {
              (quote!(Some(#label_name.to_string())), false)
            };

            Some(quote! {
              writer.set_start_event_name(#start_event);
              writer.set_skip_start_end(#skip_start);
              ::yaserde::YaSerialize::serialize(&self.#label, writer)?;
            })*/
          }
          Field::FieldVec { .. } => {
            unimplemented!();
          }
        },
      }
    })
    .collect();

  implement_serializer(
    name,
    root,
    root_attributes,
    append_attributes,
    struct_inspector,
    generics,
  )
}
