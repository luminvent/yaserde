#![allow(dead_code)]
// related to issue https://github.com/media-io/yaserde/issues/3
use yaserde::*;

#[derive(Debug, Clone, PartialEq, YaDeserialize)]
#[yaserde(rename = "layout")]
pub struct Layout {
  #[yaserde(attribute = true)]
  pub version: u32,
  #[yaserde(attribute = true)]
  pub mode: u32,
  #[yaserde(attribute = true)]
  pub w: u32,
  #[yaserde(attribute = true)]
  pub h: u32,
  #[yaserde(attribute = true)]
  pub orientation: String,
  pub tabpage: Vec<Tabpage>,
}

#[derive(Debug, Clone, PartialEq, YaDeserialize)]
pub struct Tabpage {
  #[yaserde(attribute = true, rename = "name")]
  pub named: String,
  #[yaserde(attribute = true)]
  pub scalef: f32,
  #[yaserde(attribute = true)]
  pub scalet: f32,
  #[yaserde(attribute = true)]
  pub li_t: String,
  #[yaserde(attribute = true)]
  pub li_c: String,
  #[yaserde(attribute = true)]
  pub li_s: u32,
  #[yaserde(attribute = true)]
  pub li_o: bool,
  #[yaserde(attribute = true)]
  pub li_b: bool,
  #[yaserde(attribute = true)]
  pub la_t: String,
  #[yaserde(attribute = true)]
  pub la_c: String,
  #[yaserde(attribute = true)]
  pub la_s: u32,
  #[yaserde(attribute = true)]
  pub la_o: bool,
  #[yaserde(attribute = true)]
  pub la_b: bool,
  pub control: Vec<Control>,
}

#[derive(Debug, Clone, PartialEq, YaDeserialize)]
pub struct Control {
  #[yaserde(attribute = true, rename = "name")]
  pub named: String,
  #[yaserde(attribute = true)]
  pub x: u32,
  #[yaserde(attribute = true)]
  pub y: u32,
  #[yaserde(attribute = true)]
  pub w: u32,
  #[yaserde(attribute = true)]
  pub h: u32,
  #[yaserde(attribute = true)]
  pub color: String,
  #[yaserde(attribute = true)]
  pub scalef: Option<f32>,
  #[yaserde(attribute = true)]
  pub scalet: Option<f32>,
  #[yaserde(attribute = true)]
  pub local_off: Option<bool>,
  #[yaserde(attribute = true)]
  pub sp: Option<bool>,
  #[yaserde(attribute = true)]
  pub sr: Option<bool>,
  pub midi: Vec<Midi>,
  #[yaserde(attribute = true)]
  pub response: Option<String>,
  #[yaserde(attribute = true)]
  pub inverted: Option<String>,
  #[yaserde(attribute = true)]
  pub centered: Option<String>,
  #[yaserde(attribute = true)]
  pub norollover: Option<String>,
}

#[derive(Debug, Clone, PartialEq, YaDeserialize)]
pub struct Midi {
  #[yaserde(attribute = true)]
  pub var: String,
  #[yaserde(attribute = true, rename = "type")]
  pub typ: String,
  #[yaserde(attribute = true)]
  pub channel: String,
  #[yaserde(attribute = true)]
  pub data1: String,
  #[yaserde(attribute = true)]
  pub data2f: String,
  #[yaserde(attribute = true)]
  pub data2t: String,
  #[yaserde(attribute = true)]
  pub sysex: String,
}

#[test]
fn parsing_bbigras_namespace() {
  use std::fs;
  use yaserde::de::from_str;

  let filename = "tests/data/boscop.xml";
  let content = fs::read_to_string(filename).expect("something went wrong reading the file");

  let loaded: Layout = from_str(&content).unwrap();

  assert_eq!(loaded.tabpage.len(), 4);
  assert_eq!(loaded.tabpage[0].control.len(), 13);
  assert_eq!(loaded.tabpage[1].control.len(), 16);
  assert_eq!(loaded.tabpage[2].control.len(), 65);
  assert_eq!(loaded.tabpage[3].control.len(), 40);
}
