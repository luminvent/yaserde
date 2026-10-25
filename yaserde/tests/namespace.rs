#[macro_use]
extern crate yaserde;
#[macro_use]
extern crate yaserde_derive;

fn init() {
  let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn struct_simple_namespace() {
  init();

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  #[yaserde(
    rename = "book",
    prefix = "ns",
    namespaces = {
      "ns" = "http://www.sample.com/ns/domain"
    }
  )]
  pub struct Book {
    #[yaserde(prefix = "ns")]
    author: String,
    #[yaserde(prefix = "ns")]
    title: String,
  }

  let content = r#"
    <ns:book xmlns:ns="http://www.sample.com/ns/domain">
      <ns:author>Antoine de Saint-Exupéry</ns:author>
      <ns:title>Little prince</ns:title>
    </ns:book>
  "#;

  let model = Book {
    author: "Antoine de Saint-Exupéry".to_owned(),
    title: "Little prince".to_owned(),
  };

  serialize_and_validate!(model, content);
  deserialize_and_validate!(content, model, Book);
}

#[test]
fn struct_multiple_namespaces() {
  init();

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  #[yaserde(
    rename = "book",
    prefix = "ns",
    namespaces = {
      "ns" = "http://www.sample.com/ns/domain",
      "ns2" = "http://www.sample.com/ns/domain_2"
    }
  )]
  pub struct Book {
    #[yaserde(prefix = "ns")]
    author: String,
    #[yaserde(prefix = "ns2")]
    title: String,
  }

  let content = r#"
    <ns:book xmlns:ns="http://www.sample.com/ns/domain" xmlns:ns2="http://www.sample.com/ns/domain_2">
      <ns:author>Antoine de Saint-Exupéry</ns:author>
      <ns2:title>Little prince</ns2:title>
    </ns:book>
  "#;

  let model = Book {
    author: "Antoine de Saint-Exupéry".to_owned(),
    title: "Little prince".to_owned(),
  };

  serialize_and_validate!(model, content);
  deserialize_and_validate!(content, model, Book);
}

#[test]
fn struct_partial_namespace() {
  init();

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  #[yaserde(
    rename = "book",
    prefix = "ns",
    namespaces = {
      "ns" = "http://www.sample.com/ns/domain"
    }
  )]
  pub struct Book {
    author: String,
    #[yaserde(prefix = "ns")]
    title: String,
  }

  let content = r#"
    <ns:book xmlns:ns="http://www.sample.com/ns/domain">
      <author>Antoine de Saint-Exupéry</author>
      <ns:title>Little prince</ns:title>
    </ns:book>
  "#;

  let model = Book {
    author: "Antoine de Saint-Exupéry".to_owned(),
    title: "Little prince".to_owned(),
  };

  serialize_and_validate!(model, content);
  deserialize_and_validate!(content, model, Book);
}

#[test]
fn struct_sub_namespace_definition() {
  init();

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  #[yaserde(
    rename = "book",
    prefix = "ns",
    namespaces = {
      "ns" = "http://www.sample.com/ns/domain",
      "ns2" = "http://www.sample.com/ns/domain_2"
    }
  )]
  pub struct Book {
    #[yaserde(prefix = "ns")]
    author: String,
    #[yaserde(prefix = "ns2", namespaces = {
      "ns2" = "http://www.sample.com/ns/domain_2"
    })]
    title: String,
  }

  let content = r#"
    <ns:book xmlns:ns="http://www.sample.com/ns/domain">
      <ns:author>Antoine de Saint-Exupéry</ns:author>
      <ns2:title xmlns:ns2="http://www.sample.com/ns/domain_2">Little prince</ns2:title>
    </ns:book>
  "#;

  let model = Book {
    author: "Antoine de Saint-Exupéry".to_owned(),
    title: "Little prince".to_owned(),
  };

  // TODO support namespace for attribute to specify local namespace
  // serialize_and_validate!(model, content);
  deserialize_and_validate!(content, model, Book);
}

#[test]
fn struct_namespace_nested() {
  init();

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  #[yaserde(prefix = "nsa", namespaces = {
    "nsa" = "http://www.sample.com/ns/a"
  })]
  struct A {
    #[yaserde(prefix = "nsa")]
    alpha: i32,
  }

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  #[yaserde(prefix = "nsb", namespaces = {
    "nsb" = "http://www.sample.com/ns/b"
  })]
  struct B {
    // Note that name `nested` resides in `nsb` though it has a type from `nsa`
    #[yaserde(prefix = "nsb")]
    nested: A,
  }

  let content = r#"
    <nsb:B xmlns:nsb="http://www.sample.com/ns/b">
      <nsb:nested xmlns:nsa="http://www.sample.com/ns/a">
        <nsa:alpha>32</nsa:alpha>
      </nsb:nested>
    </nsb:B>
  "#;

  let model = B {
    nested: A { alpha: 32 },
  };

  serialize_and_validate!(model, content);
  deserialize_and_validate!(content, model, B);
}

#[test]
fn struct_namespace_nested_defined_at_root() {
  init();

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  #[yaserde(prefix = "nsa", namespaces = {
    "nsa" = "http://www.sample.com/ns/a"
  })]
  struct A {
    #[yaserde(prefix = "nsa")]
    alpha: i32,
  }

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  #[yaserde(
    prefix = "nsb",
    namespaces = {
      "nsb" = "http://www.sample.com/ns/b",
      "nsa" = "http://www.sample.com/ns/a"
    }
  )]
  struct B {
    // Note that name `nested` resides in `nsb` though it has a type from `nsa`
    #[yaserde(prefix = "nsb")]
    nested: A,
  }

  let content = r#"
    <nsb:B xmlns:nsa="http://www.sample.com/ns/a" xmlns:nsb="http://www.sample.com/ns/b">
      <nsb:nested>
        <nsa:alpha>32</nsa:alpha>
      </nsb:nested>
    </nsb:B>
  "#;

  let model = B {
    nested: A { alpha: 32 },
  };

  serialize_and_validate!(model, content);
  deserialize_and_validate!(content, model, B);
}

#[test]
fn struct_attribute_namespace() {
  init();

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  #[yaserde(
    rename = "root",
    namespaces = {
      "ns1" = "http://www.sample.com/ns/domain1",
      "ns2" = "http://www.sample.com/ns/domain2"
    }
  )]
  pub struct XmlStruct {
    #[yaserde(prefix = "ns1")]
    item_1: String,
    #[yaserde(attribute = true, prefix = "ns2")]
    item_2: String,
  }

  let model = XmlStruct {
    item_1: "something 1".to_string(),
    item_2: "something 2".to_string(),
  };

  let content = r#"
    <root xmlns:ns1="http://www.sample.com/ns/domain1" xmlns:ns2="http://www.sample.com/ns/domain2" ns2:item_2="something 2">
      <ns1:item_1>something 1</ns1:item_1>
    </root>
  "#;

  serialize_and_validate!(model, content);
  deserialize_and_validate!(content, model, XmlStruct);
}

#[test]
fn struct_implicit_default_namespace() {
  init();

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  #[yaserde(
    rename = "tt",
    default_namespace = "ttml",
    namespaces = {
      "ttml" = "http://www.w3.org/ns/ttml",
      "ttm" = "http://www.w3.org/ns/ttml#metadata"
    }
  )]
  pub struct XmlStruct {
    item: String,
  }

  let model = XmlStruct {
    item: "something".to_string(),
  };

  let content = r#"<tt xmlns="http://www.w3.org/ns/ttml" xmlns:ttm="http://www.w3.org/ns/ttml#metadata"><item>something</item></tt>"#;
  serialize_and_validate!(model, content);
  // TODO
  // deserialize_and_validate!(content, model, XmlStruct);
}

#[test]
fn struct_explicit_default_namespace() {
  init();

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  #[yaserde(
    rename = "tt",
    default_namespace = "ttml",
    namespaces = {
      "ttml" = "http://www.w3.org/ns/ttml",
      "ttm" = "http://www.w3.org/ns/ttml#metadata"
    }
  )]
  pub struct XmlStruct {
    item: String,
  }

  let model = XmlStruct {
    item: "something".to_string(),
  };

  let content = r#"<tt xmlns="http://www.w3.org/ns/ttml" xmlns:ttm="http://www.w3.org/ns/ttml#metadata"><item>something</item></tt>"#;
  serialize_and_validate!(model, content);
  // TODO
  // deserialize_and_validate!(content, model, XmlStruct);
}

#[test]
fn struct_default_namespace_via_attribute_with_prefix() {
  init();

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  #[yaserde(
    rename = "tt",
    prefix = "TTML",
    default_namespace = "TTML",
    namespaces = {
      "TTML" = "http://www.w3.org/ns/ttml",
      "ttm" = "http://www.w3.org/ns/ttml#metadata"
    }
  )]
  pub struct XmlStruct {
    #[yaserde(prefix = "TTML")]
    item: String,
  }

  let model = XmlStruct {
    item: "something".to_string(),
  };

  let content = r#"<tt xmlns="http://www.w3.org/ns/ttml" xmlns:ttm="http://www.w3.org/ns/ttml#metadata"><item>something</item></tt>"#;
  serialize_and_validate!(model, content);
  deserialize_and_validate!(content, model, XmlStruct);
}

#[test]
fn enum_namespace() {
  init();

  #[derive(Debug, Default, PartialEq, YaDeserialize, YaSerialize)]
  #[yaserde(
    rename = "root",
    prefix = "ns",
    namespaces = {
      "ns" = "http://www.sample.com/ns/domain"
    }
  )]
  pub enum XmlStruct {
    #[yaserde(prefix = "ns")]
    #[default]
    Item,
  }

  let content = r#"
    <ns:root xmlns:ns="http://www.sample.com/ns/domain">
      ns:Item
    </ns:root>
  "#;

  let model = XmlStruct::Item;
  serialize_and_validate!(model, content);
  deserialize_and_validate!(content, model, XmlStruct);
}

#[test]
fn enum_multi_namespaces() {
  init();

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  #[yaserde(
    rename = "root",
    namespaces = {
    "ns1" = "http://www.sample.com/ns/domain1",
    "ns2" = "http://www.sample.com/ns/domain2"
    }
  )]
  #[derive(Default)]
  pub enum XmlStruct {
    #[yaserde(prefix = "ns1")]
    #[default]
    Item1,
    #[yaserde(prefix = "ns2")]
    Item2,
  }

  let model = XmlStruct::Item1;
  let content = r#"
    <root xmlns:ns1="http://www.sample.com/ns/domain1" xmlns:ns2="http://www.sample.com/ns/domain2">
      ns1:Item1
    </root>
  "#;
  serialize_and_validate!(model, content);
  deserialize_and_validate!(content, model, XmlStruct);

  let model = XmlStruct::Item2;
  let content = r#"
    <root xmlns:ns1="http://www.sample.com/ns/domain1" xmlns:ns2="http://www.sample.com/ns/domain2">
      ns2:Item2
    </root>
  "#;
  serialize_and_validate!(model, content);
  // TODO
  // deserialize_and_validate!(content, model, XmlStruct);
}

#[test]
fn enum_attribute_namespace() {
  init();

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  #[yaserde(
    rename = "rootA",
    prefix = "ns",
    namespaces = {
      "ns" = "http://www.sample.com/ns/domain"
    }
  )]
  #[derive(Default)]
  pub enum XmlStruct {
    #[yaserde(prefix = "ns")]
    #[default]
    Item,
    #[yaserde(prefix = "ns")]
    ItemWithField(String),
  }

  let content = r#"
    <ns:rootA xmlns:ns="http://www.sample.com/ns/domain">
      ns:Item
    </ns:rootA>
  "#;

  let model = XmlStruct::Item;
  serialize_and_validate!(model, content);
  deserialize_and_validate!(content, model, XmlStruct);

  let model = XmlStruct::ItemWithField("Value".to_string());

  let content = r#"<ns:rootA xmlns:ns="http://www.sample.com/ns/domain"><ns:ItemWithField>Value</ns:ItemWithField></ns:rootA>"#;
  serialize_and_validate!(model, content);
  deserialize_and_validate!(content, model, XmlStruct);
}

#[test]
fn struct_bad_namespace() {
  init();

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  #[yaserde(
    rename = "book",
    prefix = "ns",
    namespaces = {
      "ns" = "http://www.sample.com/ns/domain",
      "ns2" = "http://www.sample.com/ns/domain_2"
    }
  )]
  pub struct Book {
    #[yaserde(prefix = "ns")]
    author: String,
    #[yaserde(prefix = "ns2", namespaces = {
      "ns2" = "http://www.sample.com/ns/domain_2"
    })]
    title: String,
  }

  let content = r#"
    <ns:book xmlns:ns="http://www.sample.com/ns/domain2">
      <ns:author>Antoine de Saint-Exupéry</ns:author>
      <ns:title>Little prince</ns:title>
    </ns:book>
  "#;

  let loaded: Result<Book, String> = yaserde::de::from_str(content).map_err(|e| e.to_string());
  assert_eq!(
    loaded,
    Err("bad namespace for book, found http://www.sample.com/ns/domain2".to_string())
  );
}

#[test]
fn struct_default_namespace_no_prefix() {
  init();

  #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
  #[yaserde(rename = "book", prefix = "ns", default_namespace = "ns", namespaces = {
    "ns" = "http://www.sample.com/ns/domain"
  })]
  pub struct Book {
    #[yaserde(prefix = "ns")]
    author: String,
    #[yaserde(prefix = "ns")]
    title: String,
  }

  let content = r#"
    <book xmlns="http://www.sample.com/ns/domain">
      <author>Antoine de Saint-Exupéry</author>
      <title>Little prince</title>
    </book>
  "#;

  let model = Book {
    author: "Antoine de Saint-Exupéry".to_owned(),
    title: "Little prince".to_owned(),
  };

  serialize_and_validate!(model, content);
  deserialize_and_validate!(content, model, Book);
}

#[test]
fn struct_namespace_with_flatten() {
  #[derive(Debug, PartialEq, YaDeserialize)]
  struct Inner {
    value: String,
  }

  #[derive(Debug, PartialEq, YaDeserialize)]
  #[yaserde(
        rename = "root",
        prefix = "ns",
        namespaces = {
            "ns" = "http://example.com/ns"
        }
    )]
  struct Root {
    #[yaserde(flatten = true)]
    inner: Inner,
  }

  let xml = r#"
        <ns:root xmlns:ns="http://example.com/ns">
            <value>hello</value>
        </ns:root>
        "#;

  let actual: Root = yaserde::de::from_str(xml).unwrap();

  assert_eq!(
    actual,
    Root {
      inner: Inner {
        value: "hello".to_string()
      }
    }
  )
}

#[test]
fn struct_namespace_with_flatten_qualified_different_namespaces() {
  #[derive(Debug, PartialEq, YaDeserialize)]
  #[yaserde(
        namespaces = {
            "base" = "http://example.com/base"
        }
    )]
  struct Inner {
    #[yaserde(prefix = "base")]
    value: String,
  }

  #[derive(Debug, PartialEq, YaDeserialize)]
  #[yaserde(
        rename = "root",
        prefix = "ext",
        namespaces = {
            "ext" = "http://example.com/ext"
        }
    )]
  struct Root {
    #[yaserde(flatten = true)]
    inner: Inner,
  }

  let xml = r#"
        <ext:root xmlns:ext="http://example.com/ext" xmlns:base="http://example.com/base">
            <base:value>hello</base:value>
        </ext:root>
        "#;

  let actual: Root = yaserde::de::from_str(xml).unwrap();

  assert_eq!(
    actual,
    Root {
      inner: Inner {
        value: "hello".to_string()
      }
    }
  )
}
