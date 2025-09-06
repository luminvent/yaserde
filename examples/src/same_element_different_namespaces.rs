#![allow(dead_code)]
// related to issue https://github.com/media-io/yaserde/issues/186
use yaserde::*;

#[derive(YaDeserialize, Debug, PartialEq)]
#[yaserde(
  namespaces = {
    "myns" = "http://my_namespace_1/",
    "ext" = "http://my_namespace_2/",
  },
  prefix = "myns"
)]
pub struct ErrorType {
  #[yaserde(rename = "reasonCode", prefix = "myns")]
  pub reason_code: Option<u16>,
  #[yaserde(rename = "reasonCode", prefix = "ext")]
  pub ext_reason_code: Option<u16>,
}

#[test]
fn same_element_different_namespaces() {
  use yaserde::de::from_str;

  let content = r#"
    <error_type xmlns="http://my_namespace_1/" xmlns:ext="http://my_namespace_2/">
      <reasonCode>12</reasonCode>
      <ext:reasonCode>32</ext:reasonCode>
    </error_type>
  "#;

  let loaded: ErrorType = from_str(content).unwrap();
  println!("{:?}", loaded);

  let reference = ErrorType {
    reason_code: Some(12),
    ext_reason_code: Some(32),
  };

  assert_eq!(loaded, reference);
}
