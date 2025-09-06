#![allow(dead_code)]
// related to issue https://github.com/media-io/yaserde/issues/15
use yaserde::*;

#[derive(YaDeserialize, Debug, PartialEq)]
#[yaserde(
  prefix = "ss",
  namespaces = {
    "x" = "urn:schemas-microsoft-com:office:excel",
    "ss" = "urn:schemas-microsoft-com:office:spreadsheet",
    "o" = "urn:schemas-microsoft-com:office:office",
    "html" = "http://www.w3.org/TR/REC-html40"
  }
)]
struct Workbook {
  #[yaserde(rename = "Worksheet", prefix = "ss")]
  worksheet: Worksheet,
}

#[derive(YaDeserialize, Debug, PartialEq)]
#[yaserde(
  prefix = "ss",
  namespaces = {
    "x" = "urn:schemas-microsoft-com:office:excel",
    "ss" = "urn:schemas-microsoft-com:office:spreadsheet",
    "o" = "urn:schemas-microsoft-com:office:office",
    "html" = "http://www.w3.org/TR/REC-html40"
  }
)]
struct Worksheet {
  #[yaserde(rename = "Table", prefix = "ss")]
  table: Table,
  #[yaserde(attribute = true, rename = "Name", prefix = "ss")]
  ws_name: String,
}

#[derive(YaDeserialize, Debug, PartialEq)]
#[yaserde(
  prefix = "ss",
  namespaces = {
    "x" = "urn:schemas-microsoft-com:office:excel",
    "ss" = "urn:schemas-microsoft-com:office:spreadsheet",
    "o" = "urn:schemas-microsoft-com:office:office",
    "html" = "http://www.w3.org/TR/REC-html40"
  }
)]
struct Table {
  #[yaserde(attribute = true, rename = "ExpandedColumnCount", prefix = "ss")]
  expanded_column_count: u32,
  #[yaserde(attribute = true, rename = "ExpandedRowCount", prefix = "ss")]
  expanded_row_count: u32,
  #[yaserde(attribute = true, rename = "FullColumns", prefix = "x")]
  full_columns: u32,
  #[yaserde(attribute = true, rename = "FullRows", prefix = "x")]
  full_rows: u32,
  #[yaserde(attribute = true, rename = "StyleID", prefix = "ss")]
  style_id: String,
  #[yaserde(attribute = true, rename = "DefaultColumnWidth", prefix = "ss")]
  default_column_width: f32,
  #[yaserde(attribute = true, rename = "DefaultRowHeight", prefix = "ss")]
  default_column_height: f32,

  #[yaserde(rename = "Row", prefix = "ss")]
  rows: Vec<Row>,
}

#[derive(YaDeserialize, Debug, PartialEq)]
#[yaserde(
  prefix = "ss",
  namespaces = {
    "x" = "urn:schemas-microsoft-com:office:excel",
    "ss" = "urn:schemas-microsoft-com:office:spreadsheet",
    "o" = "urn:schemas-microsoft-com:office:office",
    "html" = "http://www.w3.org/TR/REC-html40"
  }
)]
struct Row {
  #[yaserde(attribute = true, rename = "AutoFitHeight", prefix = "ss")]
  auto_fit_height: f32,
  #[yaserde(attribute = true, rename = "Height", prefix = "ss")]
  height: f32,
}

#[test]
fn parsing_bbigras_namespace() {
  use std::fs;
  use yaserde::de::from_str;

  let filename = "tests/data/bbigras-namespace.xml";

  let content = fs::read_to_string(filename).expect("something went wrong reading the file");

  let loaded: Workbook = from_str(&content).unwrap();
  println!("{:?}", loaded);

  let reference = Workbook {
    worksheet: Worksheet {
      ws_name: "some_name".to_string(),
      table: Table {
        expanded_column_count: 11,
        expanded_row_count: 195,
        full_columns: 1,
        full_rows: 1,
        style_id: "s64".to_string(),
        default_column_width: 60.75,
        default_column_height: 15.0,
        rows: vec![Row {
          auto_fit_height: 0.0,
          height: 33.0,
        }],
      },
    },
  };

  assert_eq!(loaded, reference);
}
