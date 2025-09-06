#![allow(dead_code)]
// related to issue https://github.com/media-io/yaserde/issues/11
use yaserde::*;

#[derive(YaDeserialize, Debug, PartialEq)]
#[yaserde(rename = "DOMSymbolItem")]
struct Level {
  #[yaserde(attribute = true)]
  last_modified: u64,
  #[yaserde(attribute = true, rename = "name")]
  named: String,
  timeline: Timeline,
}

#[derive(YaDeserialize, Debug, PartialEq)]
struct Timeline {
  #[yaserde(rename = "DOMTimeline")]
  timeline: DOMTimeline,
}

#[derive(YaDeserialize, Debug, PartialEq)]
struct DOMTimeline {
  #[yaserde(attribute = true, rename = "name")]
  named: String,
  #[yaserde(attribute = true)]
  current_frame: u64,
  #[yaserde(attribute = true)]
  guides: u64,
  layers: Layers,
}

#[derive(YaDeserialize, Debug, PartialEq)]
struct Layers {
  #[yaserde(rename = "DOMLayer")]
  dom_layer: Vec<DOMLayer>,
}

#[derive(YaDeserialize, Debug, PartialEq)]
struct DOMLayer {
  #[yaserde(attribute = true, rename = "name")]
  named: String,
  #[yaserde(attribute = true)]
  name2: String,
}

#[test]
fn parsing_ln_dom() {
  use std::fs;
  use yaserde::de::from_str;

  let filename = "tests/data/ln-dom.xml";

  let content = fs::read_to_string(filename).expect("something went wrong reading the file");

  let loaded: Level = from_str(&content).unwrap();
  println!("{:?}", loaded);

  let reference = Level {
    last_modified: 1414141442,
    named: "dagger".to_string(),
    timeline: Timeline {
      timeline: DOMTimeline {
        named: "dagger timeline name".to_string(),
        current_frame: 7,
        guides: 11,
        layers: Layers {
          dom_layer: vec![
            DOMLayer {
              named: "Layer 2".to_string(),
              name2: "Lalayer 2".to_string(),
            },
            DOMLayer {
              named: "Layer 1".to_string(),
              name2: "Lalayer 1".to_string(),
            },
          ],
        },
      },
    },
  };

  assert_eq!(loaded, reference);
}
