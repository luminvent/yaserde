//! # YaSerDe
//!
//! YaSerDe is a framework for ***ser***ializing and ***de***serializing Rust data
//! structures efficiently and generically from and into XML.
//!
//! YaSerDe makes it easy to serialize XML documents given an properly annotated struct.
//! Please refer to the `examples` directory for the complete code shown below.
//!
//! # Serialize
//!
//! For instance, let's say that one wants to generate a XML file for the
//! [Rust-Embedded community](https://github.com/rust-embedded/). A well known XML
//! file for microcontrollers is called [SVD](https://github.com/rust-embedded/svd/)
//! and it can be defined on YaSerDe via structs like so:
//!
//!```rust
//! use yaserde_derive::YaSerialize;
//!
//! #[derive(PartialEq, Debug, YaSerialize)]
//! #[yaserde(rename = "device")]
//! struct Device {
//!   #[yaserde(attribute = true)]
//!   schemaversion: String,
//!   #[yaserde(attribute = true)]
//!   xmlns: String,
//!   #[yaserde(attribute = true)]
//!   xsnonamespaceschemalocation: String,
//!   attributes: DeviceAttributes
//! }
//!
//! #[derive(PartialEq, Debug, YaSerialize)]
//! struct DeviceAttributes {
//!   vendor: String,
//! }
//!```
//!
//! The interspersed `#[yaserde()]` macros give some indication of what the resulting XML
//! Will look like, namely, a short snippet of the struct above in XML would be depending on
//! concrete values passed to the struct (not shown):
//!
//!```xml
//! (...)
//! <device schemaversion: "1.0-example", xmlns: "ns:.... example"
//! xsnonamespaceschemalocation: "foo_bar_baz">
//!    <devattributes>
//!    </devattributes>
//! (...)
//!```
//!
//! Notice the important difference in **XML output representation between `attributes` vs
//! `child`**, since SVD expects information in that particular arrangement. YaSerDe allows that
//! serialized XML to be valid unlike other Rust XML (de)serialization crates (i.e quick-xml).
//!
//! Also the last `DevAttrs` struct field is indeed another struct, so one can chain several
//! structs to compose the XML structure (again, see examples folder for the complete
//! example).
//!
//!
//!```toml
//! [dependencies]
//! # serde = { version = "1.0.123", features = [ "derive" ] }
//! # quick-xml = { version = "0.21.0", features = [ "serialize" ] }
//! yaserde = "0.5.1"
//! yaserde_derive = "0.5.1"
//! ```
//!
//! Last but not least, in order to have a nice, pretty printed XML output one can do:
//!
//! ```ignore
//!     // Display pretty printed XML
//!    let yaserde_cfg = yaserde::ser::Config{
//!        perform_indent: true,
//!        .. Default::default()
//!    };
//!
//!     println!("{}", yaserde::ser::to_string_with_config(&dev, &yaserde_cfg).ok().unwrap());
//! ```
//!
//! Avoid using either `{:?}` or `{:#?}` println! formatters since it'll garble the output of your
//! XML.

#[cfg(feature = "yaserde_derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate yaserde_derive;

#[cfg(feature = "yaserde_derive")]
#[doc(hidden)]
pub use yaserde_derive::*;

use std::io::{Read, Write};
use xml::writer::XmlEvent;

pub mod de;
pub mod primitives;
pub mod ser;

/// A **data structure** that can be deserialized from any data format supported by YaSerDe.
pub trait YaDeserialize: Sized {
  fn deserialize<R: Read>(reader: &mut de::Deserializer<R>) -> Result<Self, String>;
}

/// A **data structure** that can be serialized into any data format supported by YaSerDe.
pub trait YaSerialize: Sized {
  fn serialize<W: Write>(&self, writer: &mut ser::Serializer<W>) -> Result<(), String>;

  fn serialize_attributes(
    &self,
    attributes: Vec<xml::attribute::OwnedAttribute>,
    namespace: xml::namespace::Namespace,
  ) -> Result<
    (
      Vec<xml::attribute::OwnedAttribute>,
      xml::namespace::Namespace,
    ),
    String,
  >;
}

/// A **visitor** that can be implemented to retrieve information from source file.
pub trait Visitor<'de>: Sized {
  /// The value produced by this visitor.
  type Value;

  fn visit_bool(self, v: &str) -> Result<Self::Value, String> {
    Err(format!("Unexpected bool {:?}", v))
  }

  fn visit_i8(self, v: &str) -> Result<Self::Value, String> {
    Err(format!("Unexpected i8 {:?}", v))
  }

  fn visit_u8(self, v: &str) -> Result<Self::Value, String> {
    Err(format!("Unexpected u8 {:?}", v))
  }

  fn visit_i16(self, v: &str) -> Result<Self::Value, String> {
    Err(format!("Unexpected i16 {:?}", v))
  }

  fn visit_u16(self, v: &str) -> Result<Self::Value, String> {
    Err(format!("Unexpected u16 {:?}", v))
  }

  fn visit_i32(self, v: &str) -> Result<Self::Value, String> {
    Err(format!("Unexpected i32 {:?}", v))
  }

  fn visit_u32(self, v: &str) -> Result<Self::Value, String> {
    Err(format!("Unexpected u32 {:?}", v))
  }

  fn visit_i64(self, v: &str) -> Result<Self::Value, String> {
    Err(format!("Unexpected i64 {:?}", v))
  }

  fn visit_u64(self, v: &str) -> Result<Self::Value, String> {
    Err(format!("Unexpected u64 {:?}", v))
  }

  fn visit_f32(self, v: &str) -> Result<Self::Value, String> {
    Err(format!("Unexpected f32 {:?}", v))
  }

  fn visit_f64(self, v: &str) -> Result<Self::Value, String> {
    Err(format!("Unexpected f64 {:?}", v))
  }

  fn visit_str(self, v: &str) -> Result<Self::Value, String> {
    Err(format!("Unexpected str {:?}", v))
  }
}

macro_rules! serialize_type {
  ($type:ty) => {
    impl YaSerialize for $type {
      fn serialize<W: Write>(&self, writer: &mut ser::Serializer<W>) -> Result<(), String> {
        let content = format!("{}", self);
        let event = XmlEvent::characters(&content);
        let _ret = writer.write(event);
        Ok(())
      }

      fn serialize_attributes(
        &self,
        attributes: Vec<xml::attribute::OwnedAttribute>,
        namespace: xml::namespace::Namespace,
      ) -> Result<
        (
          Vec<xml::attribute::OwnedAttribute>,
          xml::namespace::Namespace,
        ),
        String,
      > {
        Ok((attributes, namespace))
      }
    }
  };
}

serialize_type!(bool);
serialize_type!(char);

serialize_type!(usize);
serialize_type!(u8);
serialize_type!(u16);
serialize_type!(u32);
serialize_type!(u64);

serialize_type!(isize);
serialize_type!(i8);
serialize_type!(i16);
serialize_type!(i32);
serialize_type!(i64);

serialize_type!(f32);
serialize_type!(f64);

/// Re-export for use in yaserde_derive
#[doc(hidden)]
pub use xml as __xml;

/// Re-export for use in yaserde_derive
#[doc(hidden)]
pub use log as __log;

/// Used in code generated by yaserde_derive for logging
#[macro_export]
#[doc(hidden)]
macro_rules! __derive_debug {
  ($($arg:tt)+) => { ::yaserde::__log::debug!(target: "yaserde_derive", $($arg)+) };
}

/// Used in code generated by yaserde_derive for logging
#[macro_export]
#[doc(hidden)]
macro_rules! __derive_trace {
  ($($arg:tt)+) => { ::yaserde::__log::trace!(target: "yaserde_derive", $($arg)+) };
}

#[test]
fn default_visitor() {
  struct Test;
  impl<'de> Visitor<'de> for Test {
    type Value = u8;
  }

  macro_rules! test_type {
    ($visitor:tt, $message:expr) => {{
      let t = Test {};
      assert_eq!(t.$visitor(""), Err($message.to_string()));
    }};
  }

  test_type!(visit_bool, "Unexpected bool \"\"");
  test_type!(visit_i8, "Unexpected i8 \"\"");
  test_type!(visit_u8, "Unexpected u8 \"\"");
  test_type!(visit_i16, "Unexpected i16 \"\"");
  test_type!(visit_u16, "Unexpected u16 \"\"");
  test_type!(visit_i32, "Unexpected i32 \"\"");
  test_type!(visit_u32, "Unexpected u32 \"\"");
  test_type!(visit_i64, "Unexpected i64 \"\"");
  test_type!(visit_u64, "Unexpected u64 \"\"");
  test_type!(visit_str, "Unexpected str \"\"");
}

#[doc(hidden)]
#[macro_export]
macro_rules! test_for_type {
  ($type:ty, $value:expr, $content:expr) => {{
    #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
    #[yaserde(rename = "data")]
    pub struct Data {
      item: $type,
    }

    let model = Data { item: $value };

    let content = if let Some(str_value) = $content {
      let str_value: &str = str_value;
      format!("<data><item>{}</item></data>", str_value)
    } else {
      "<data />".to_owned()
    };

    serialize_and_validate!(model, content);
    deserialize_and_validate!(&content, model, Data);
  }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! test_for_attribute_type {
  ($type: ty, $value: expr, $content: expr) => {{
    #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
    #[yaserde(rename = "data")]
    pub struct Data {
      #[yaserde(attribute = true)]
      item: $type,
    }
    let model = Data { item: $value };

    let content = if let Some(str_value) = $content {
      "<data item=\"".to_string() + str_value + "\" />"
    } else {
      "<data />".to_string()
    };

    serialize_and_validate!(model, content);
    deserialize_and_validate!(&content, model, Data);
  }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! deserialize_and_validate {
  ($content: expr, $model: expr, $struct: tt) => {
    log::debug!("deserialize_and_validate @ {}:{}", file!(), line!());
    let loaded: Result<$struct, String> = yaserde::de::from_str($content);
    assert_eq!(loaded, Ok($model));
  };
}

#[doc(hidden)]
#[macro_export]
macro_rules! serialize_and_validate {
  ($model: expr, $content: expr) => {
    log::debug!("serialize_and_validate @ {}:{}", file!(), line!());
    let data: Result<String, String> = yaserde::ser::to_string(&$model);

    let content = &format!(r#"<?xml version="1.0" encoding="UTF-8"?>{}"#, $content);
    assert_eq!(
      data,
      Ok(content.split("\n").map(|s| s.trim()).collect::<String>())
    );
  };
}
