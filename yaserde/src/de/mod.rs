//! Generic data structure deserialization framework.
//!

use crate::YaDeserialize;
use std::io::Read;
use xml::name::OwnedName;
use xml::reader::{EventReader, ParserConfig, XmlEvent};

pub fn from_str<T: YaDeserialize>(s: &str) -> Result<T, String> {
  from_reader(s.as_bytes())
}

pub fn from_reader<R: Read, T: YaDeserialize>(reader: R) -> Result<T, String> {
  <T as YaDeserialize>::deserialize(&mut Deserializer::new_from_reader(reader))
}

pub struct Deserializer<R: Read> {
  depth: usize,
  reader: EventReader<R>,
  peeked: Option<XmlEvent>,
  pub inner_struct_label: Option<&'static str>,
}

impl<R: Read> Deserializer<R> {
  pub fn new(reader: EventReader<R>) -> Self {
    Deserializer {
      depth: 0,
      reader,
      peeked: None,
      inner_struct_label: None,
    }
  }

  pub fn new_from_reader(reader: R) -> Self {
    let config = ParserConfig::new()
      .trim_whitespace(true)
      .whitespace_to_characters(true)
      .cdata_to_characters(true)
      .ignore_comments(true)
      .coalesce_characters(true);

    Self::new(EventReader::new_with_config(reader, config))
  }

  pub fn peek(&mut self) -> Result<&XmlEvent, String> {
    if self.peeked.is_none() {
      self.peeked = Some(self.inner_next()?);
    }

    if let Some(ref next) = self.peeked {
      Ok(next)
    } else {
      Err("unable to peek next item".into())
    }
  }

  pub fn inner_next(&mut self) -> Result<XmlEvent, String> {
    loop {
      match self.reader.next() {
        Ok(next) => {
          match next {
            XmlEvent::StartDocument { .. }
            | XmlEvent::ProcessingInstruction { .. }
            | XmlEvent::Comment(_) => { /* skip */ }
            other => return Ok(other),
          }
        }
        Err(msg) => {
          return Err(msg.msg().to_string());
        }
      }
    }
  }

  pub fn next_event(&mut self) -> Result<XmlEvent, String> {
    let next_event = if let Some(peeked) = self.peeked.take() {
      peeked
    } else {
      self.inner_next()?
    };
    match next_event {
      XmlEvent::StartElement { .. } => {
        self.depth += 1;
      }
      XmlEvent::EndElement { .. } => {
        self.depth -= 1;
      }
      _ => {}
    }
    log::debug!("Fetched {:?}, new depth {}", next_event, self.depth);
    Ok(next_event)
  }

  pub fn skip_element(&mut self, mut cb: impl FnMut(&XmlEvent)) -> Result<(), String> {
    let depth = self.depth;

    while self.depth >= depth {
      cb(&self.next_event()?);
    }

    Ok(())
  }

  pub fn depth(&self) -> usize {
    self.depth
  }

  pub fn read_inner_value<T, F: FnOnce(&mut Self) -> Result<T, String>>(
    &mut self,
    f: F,
  ) -> Result<T, String> {
    if let Ok(XmlEvent::StartElement { name, .. }) = self.next_event() {
      let result = f(self)?;
      self.expect_end_element(&name)?;
      Ok(result)
    } else {
      Err("Internal error: Bad Event".to_string())
    }
  }

  pub fn expect_end_element(&mut self, start_name: &OwnedName) -> Result<(), String> {
    if let XmlEvent::EndElement { name, .. } = self.next_event()? {
      if name == *start_name {
        Ok(())
      } else {
        Err(format!(
          "End tag </{}> didn't match the start tag <{}>",
          name.local_name, start_name.local_name
        ))
      }
    } else {
      Err(format!("Unexpected token </{}>", start_name.local_name))
    }
  }
}
