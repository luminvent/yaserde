//! Generic data structure serialization framework.
//!

use crate::YaSerialize;
use std::io::{Cursor, Write};
use std::str;
use xml::reader::XmlEvent as ReaderEvent;
use xml::writer::XmlEvent;
use xml::{EmitterConfig, EventWriter};

/// Serialize XML into a plain String with no formatting (EmitterConfig).
pub fn to_string<T: YaSerialize>(model: &T) -> Result<String, String> {
  let buf = Cursor::new(Vec::new());
  let cursor = serialize_with_writer(model, buf, &Config::default())?;
  let data = str::from_utf8(cursor.get_ref()).expect("Found invalid UTF-8");
  Ok(data.into())
}

/// Serialize XML into a plain String with control on formatting (via EmitterConfig parameters)
pub fn to_string_with_config<T: YaSerialize>(model: &T, config: &Config) -> Result<String, String> {
  let buf = Cursor::new(Vec::new());
  let cursor = serialize_with_writer(model, buf, config)?;
  let data = str::from_utf8(cursor.get_ref()).expect("Found invalid UTF-8");
  Ok(data.into())
}

pub fn serialize_with_writer<W: Write, T: YaSerialize>(
  model: &T,
  writer: W,
  config: &Config,
) -> Result<W, String> {
  let mut serializer = Serializer::new_from_writer(writer, config);
  match YaSerialize::serialize(model, &mut serializer) {
    Ok(()) => Ok(serializer.into_inner()),
    Err(msg) => Err(msg),
  }
}

pub fn to_string_content<T: YaSerialize>(model: &T) -> Result<String, String> {
  let buf = Cursor::new(Vec::new());
  let cursor = serialize_with_writer_content(model, buf)?;
  let data = str::from_utf8(cursor.get_ref()).expect("Found invalid UTF-8");
  let wrapped = format!("<yaserde-content>{data}</yaserde-content>");
  let mut content = String::new();
  for event in xml::EventReader::from_str(&wrapped) {
    match event.map_err(|error| error.to_string())? {
      ReaderEvent::Characters(value)
      | ReaderEvent::CData(value)
      | ReaderEvent::Whitespace(value) => content.push_str(&value),
      ReaderEvent::StartElement { name, .. } if name.local_name != "yaserde-content" => {
        return Err("attribute serializers must produce text content".into());
      }
      _ => {}
    }
  }
  Ok(content)
}

pub fn serialize_with_writer_content<W: Write, T: YaSerialize>(
  model: &T,
  writer: W,
) -> Result<W, String> {
  let mut serializer = Serializer::new_for_inner(writer);
  serializer.set_skip_start_end(true);
  match YaSerialize::serialize(model, &mut serializer) {
    Ok(()) => Ok(serializer.into_inner()),
    Err(msg) => Err(msg),
  }
}

pub struct Serializer<W: Write> {
  writer: EventWriter<W>,
  skip_start_end: bool,
  start_event_name: Option<String>,
}

impl<W: Write> Serializer<W> {
  pub fn new(writer: EventWriter<W>) -> Self {
    Serializer {
      writer,
      skip_start_end: false,
      start_event_name: None,
    }
  }

  pub fn new_from_writer(writer: W, config: &Config) -> Self {
    let mut emitter_config = EmitterConfig::new()
      .cdata_to_characters(false)
      .perform_indent(config.perform_indent)
      .write_document_declaration(config.write_document_declaration);

    if let Some(indent_string_value) = &config.indent_string {
      emitter_config = emitter_config.indent_string(indent_string_value.clone());
    }

    Self::new(EventWriter::new_with_config(writer, emitter_config))
  }

  pub fn new_for_inner(writer: W) -> Self {
    let config = EmitterConfig::new().write_document_declaration(false);

    Self::new(EventWriter::new_with_config(writer, config))
  }

  pub fn into_inner(self) -> W {
    self.writer.into_inner()
  }

  pub fn skip_start_end(&self) -> bool {
    self.skip_start_end
  }

  pub fn set_skip_start_end(&mut self, state: bool) {
    self.skip_start_end = state;
  }

  pub fn get_start_event_name(&self) -> Option<String> {
    self.start_event_name.clone()
  }

  pub fn set_start_event_name(&mut self, name: Option<String>) {
    self.start_event_name = name;
  }

  pub fn write<'a, E>(&mut self, event: E) -> xml::writer::Result<()>
  where
    E: Into<XmlEvent<'a>>,
  {
    self.writer.write(event)
  }
}

pub struct Config {
  pub perform_indent: bool,
  pub write_document_declaration: bool,
  pub indent_string: Option<String>,
}

impl Default for Config {
  fn default() -> Self {
    Config {
      perform_indent: false,
      write_document_declaration: true,
      indent_string: None,
    }
  }
}
