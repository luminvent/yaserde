#![allow(dead_code)]
use yaserde::*;

#[derive(YaSerialize, YaDeserialize, Debug, Default, Clone, Eq, PartialEq)]
pub struct Header {}

#[derive(YaSerialize, YaDeserialize, Debug, Default, Clone, Eq, PartialEq)]
#[yaserde(
  rename = "Envelope",
  namespaces = {
    "s" = "http://schemas.xmlsoap.org/soap/envelope/",
  },
  prefix = "s"
)]
pub struct SoapEnvelope<BODY>
where
  BODY: YaSerialize + YaDeserialize + Default,
{
  #[yaserde(rename = "encodingStyle", prefix = "s", attribute = true)]
  pub encoding_style: String,
  #[yaserde(rename = "u", prefix = "xmlns", attribute = true)]
  pub tnsattr: Option<String>,
  #[yaserde(rename = "urn", prefix = "xmlns", attribute = true)]
  pub urnattr: Option<String>,
  #[yaserde(rename = "xsi", prefix = "xmlns", attribute = true)]
  pub xsiattr: Option<String>,
  #[yaserde(rename = "Header", prefix = "s")]
  pub header: Option<Header>,
  #[yaserde(rename = "Body", prefix = "s")]
  pub body: BODY,
}

#[derive(YaSerialize, YaDeserialize, Debug, Default, Clone, Eq, PartialEq)]
#[yaserde(namespaces = {
  "u" = "urn:schemas-upnp-org:service:AVTransport:1"
})]
pub struct SoapPlay {
  #[yaserde(rename = "Play", prefix = "u", default = "default_play")]
  pub body: Play,
}

fn default_play() -> Play {
  Play::default()
}

#[derive(YaSerialize, YaDeserialize, Debug, Default, Clone, Eq, PartialEq)]
#[yaserde(rename = "Play", prefix = "u")]
pub struct Play {
  #[yaserde(flatten = true, default = "default_play2")]
  pub parameters: Play2,
}

fn default_play2() -> Play2 {
  Play2::default()
}

#[derive(YaSerialize, YaDeserialize, Debug, Default, Clone, Eq, PartialEq)]
#[yaserde(
  rename = "Play",
  namespaces = {
    "u" = "urn:schemas-upnp-org:service:AVTransport:1",
  },
  prefix = "u"
)]
pub struct Play2 {
  #[yaserde(rename = "InstanceID", default = "default_i32")]
  pub instance_id: i32,
  #[yaserde(rename = "Speed", default = "default_i32")]
  pub speed: i32,
}

fn default_i32() -> i32 {
  i32::default()
}

#[derive(PrimitiveYaSerde, Debug, Default, Eq, PartialEq)]
struct Meters(i32);

#[test]
fn test_for_generic_newtype() {
  let a = SoapEnvelope {
    encoding_style: "".to_string(),
    tnsattr: None,
    urnattr: None,
    xsiattr: None,
    header: None,
    body: Meters(10),
  };

  let s = ser::to_string(&a).unwrap();
  let b: SoapEnvelope<Meters> = de::from_str(&s).unwrap();

  assert_eq!(a, b);
  println!("{:#?}", b);
}

#[test]
fn test_for_generic_nested_struct() {
  let a = SoapEnvelope {
    encoding_style: "".to_string(),
    tnsattr: None,
    urnattr: None,
    xsiattr: None,
    header: None,
    body: SoapPlay {
      body: Play {
        parameters: Play2 {
          instance_id: 20,
          speed: 1,
        },
      },
    },
  };

  let s = ser::to_string(&a).unwrap();
  println!("{s}");
  let b: SoapEnvelope<SoapPlay> = de::from_str(&s).unwrap();

  assert_eq!(a, b);
  println!("{:#?}", b);
}
