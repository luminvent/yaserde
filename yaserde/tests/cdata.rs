use yaserde_derive::{YaDeserialize, YaSerialize};

fn init() {
  let _ = env_logger::builder().is_test(true).try_init();
}

#[derive(YaSerialize, YaDeserialize, PartialEq, Debug)]
#[yaserde(rename = "teststruct")]
struct TestStruct {
  #[yaserde(cdata = true)]
  pub msgdata: String,
}

#[test]
fn test_cdata_serialization() {
  init();
  let test_data = TestStruct {
    msgdata: "<tag>Some unescaped content</tag>".to_string(),
  };
  let xml_output = yaserde::ser::to_string(&test_data).expect("Serialization failed");
  let expected_output = r#"<?xml version="1.0" encoding="UTF-8"?><teststruct><msgdata><![CDATA[<tag>Some unescaped content</tag>]]></msgdata></teststruct>"#;
  assert_eq!(xml_output, expected_output);
}

#[test]
fn test_cdata_deserialization() {
  init();
  let xml = r#"<?xml version="1.0" encoding="UTF-8"?><teststruct><msgdata><![CDATA[<tag>Some unescaped content</tag>]]></msgdata></teststruct>"#;
  let r: TestStruct = yaserde::de::from_str(xml).unwrap();
  let expected_output = TestStruct {
    msgdata: "<tag>Some unescaped content</tag>".to_string(),
  };
  assert_eq!(r, expected_output);
}
