#[macro_use]
extern crate yaserde;
#[macro_use]
extern crate yaserde_derive;

fn init() {
  let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn basic_option_types() {
  init();

  test_for_type!(Option::<String>, Some("test".to_string()), Some("test"));
  test_for_type!(Option::<String>, None, None);
  test_for_type!(Option::<bool>, Some(true), Some("true"));
  test_for_type!(Option::<bool>, None, None);
  test_for_type!(Option::<u8>, Some(12_u8), Some("12"));
  test_for_type!(Option::<u8>, None, None);
  test_for_type!(Option::<i8>, Some(12_i8), Some("12"));
  test_for_type!(Option::<i8>, Some(-12_i8), Some("-12"));
  test_for_type!(Option::<i8>, None, None);
  test_for_type!(Option::<u16>, Some(12_u16), Some("12"));
  test_for_type!(Option::<u16>, None, None);
  test_for_type!(Option::<i16>, Some(12_i16), Some("12"));
  test_for_type!(Option::<i16>, Some(-12_i16), Some("-12"));
  test_for_type!(Option::<i16>, None, None);
  test_for_type!(Option::<u32>, Some(12_u32), Some("12"));
  test_for_type!(Option::<u32>, None, None);
  test_for_type!(Option::<i32>, Some(12_i32), Some("12"));
  test_for_type!(Option::<i32>, Some(-12_i32), Some("-12"));
  test_for_type!(Option::<i32>, None, None);
  test_for_type!(Option::<u64>, Some(12_u64), Some("12"));
  test_for_type!(Option::<u64>, None, None);
  test_for_type!(Option::<i64>, Some(12_i64), Some("12"));
  test_for_type!(Option::<i64>, Some(-12_i64), Some("-12"));
  test_for_type!(Option::<i64>, None, None);
  test_for_type!(Option::<f32>, Some(-12.5_f32), Some("-12.5"));
  test_for_type!(Option::<f32>, None, None);
  test_for_type!(Option::<f64>, Some(-12.5_f64), Some("-12.5"));
  test_for_type!(Option::<f64>, None, None);

  // TODO
  // test_for_type!(Option::<Vec::<u8>>, None, None);
  // test_for_type!(Option::<Vec::<u8>>, Some(vec![0]), Some("0"));
  // test_for_type!(Option::<Vec::<String>>, None, None);
  // test_for_type!(Option::<Vec::<String>>, Some(vec!["test".to_string()]), Some("test"));

  test_for_attribute_type!(Option::<String>, Some("test".to_string()), Some("test"));
  test_for_attribute_type!(Option::<String>, None, None);
  test_for_attribute_type!(Option::<bool>, Some(true), Some("true"));
  test_for_attribute_type!(Option::<bool>, None, None);
  test_for_attribute_type!(Option::<u8>, Some(12_u8), Some("12"));
  test_for_attribute_type!(Option::<u8>, None, None);
  test_for_attribute_type!(Option::<i8>, Some(12_i8), Some("12"));
  test_for_attribute_type!(Option::<i8>, Some(-12_i8), Some("-12"));
  test_for_attribute_type!(Option::<i8>, None, None);
  test_for_attribute_type!(Option::<u16>, Some(12_u16), Some("12"));
  test_for_attribute_type!(Option::<u16>, None, None);
  test_for_attribute_type!(Option::<i16>, Some(12_i16), Some("12"));
  test_for_attribute_type!(Option::<i16>, Some(-12_i16), Some("-12"));
  test_for_attribute_type!(Option::<i16>, None, None);
  test_for_attribute_type!(Option::<u32>, Some(12_u32), Some("12"));
  test_for_attribute_type!(Option::<u32>, None, None);
  test_for_attribute_type!(Option::<i32>, Some(12_i32), Some("12"));
  test_for_attribute_type!(Option::<i32>, Some(-12_i32), Some("-12"));
  test_for_attribute_type!(Option::<i32>, None, None);
  test_for_attribute_type!(Option::<u64>, Some(12_u64), Some("12"));
  test_for_attribute_type!(Option::<u64>, None, None);
  test_for_attribute_type!(Option::<i64>, Some(12_i64), Some("12"));
  test_for_attribute_type!(Option::<i64>, Some(-12_i64), Some("-12"));
  test_for_attribute_type!(Option::<i64>, None, None);
  test_for_attribute_type!(Option::<f32>, Some(-12.5_f32), Some("-12.5"));
  test_for_attribute_type!(Option::<f32>, None, None);
  test_for_attribute_type!(Option::<f64>, Some(-12.5_f64), Some("-12.5"));
  test_for_attribute_type!(Option::<f64>, None, None);
}

#[test]
fn option_struct() {
  init();

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  struct Test {
    field: SubTest,
  }

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  struct SubTest {
    content: Option<String>,
  }

  test_for_type!(
    Option::<Test>,
    Some(Test {
      field: SubTest {
        content: Some("value".to_string())
      }
    }),
    Some("<field><content>value</content></field>")
  );
  test_for_type!(Option::<Test>, None, None);
}

#[test]
fn option_bool_no_crash_on_bad_input() {
  init();

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  struct Test {
    field: SubTest,
  }

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  struct SubTest {
    #[yaserde(attribute = true)]
    content: Option<bool>,
  }

  let content = "<field><content>/<R/";
  let result: Result<Test, String> = yaserde::de::from_str(content);

  assert!(result.is_err());
}

#[cfg(test)]
mod tests {
  use super::*;
  use yaserde::de::from_str;

  #[derive(Debug, YaDeserialize, YaSerialize)]
  pub struct Car {
    #[yaserde(rename = "CarColor")]
    color: String,
    #[yaserde(rename = "CarBrand")]
    brand: String,
  }

  #[derive(Debug, YaDeserialize, YaSerialize)]
  pub struct Person {
    #[yaserde(flatten = true)]
    pub car: Option<Car>,
  }

  #[test]
  fn deserialize_without_car() {
    let person = r#"<?xml version="1.0" encoding="UTF-8"?>
      <Person>
          <EyeColor>brown</EyeColor>
          <Age>25</Age>
          <HasHome>true</HasHome>
          <HasGarden>false</HasGarden>
      </Person>
    "#;

    let person: Person = from_str(person).unwrap();

    let expected_person = Person { car: None };

    assert_eq!(person.car.is_none(), expected_person.car.is_none());
  }
}
