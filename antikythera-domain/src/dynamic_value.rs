//! Domain-native dynamic value type.
//!
//! Replaces serde_json::Value in domain entities so the domain ring
//! has zero dependency on serialization frameworks.

use serde::{Deserialize, Serialize};

/// A dynamic value that can represent any JSON-like structure.
/// This is the domain's own representation — adapters convert
/// to/from serde_json::Value at the infrastructure boundary.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DynamicValue {
    #[default]
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<DynamicValue>),
    Object(Vec<(String, DynamicValue)>),
}

impl DynamicValue {
    pub fn is_null(&self) -> bool {
        matches!(self, DynamicValue::Null)
    }
    pub fn as_str(&self) -> Option<&str> {
        if let DynamicValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        if let DynamicValue::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        if let DynamicValue::Bool(b) = self {
            Some(*b)
        } else {
            None
        }
    }
    pub fn as_array(&self) -> Option<&[DynamicValue]> {
        if let DynamicValue::Array(a) = self {
            Some(a)
        } else {
            None
        }
    }
    pub fn as_object(&self) -> Option<&[(String, DynamicValue)]> {
        if let DynamicValue::Object(o) = self {
            Some(o)
        } else {
            None
        }
    }

    pub fn is_array(&self) -> bool {
        matches!(self, DynamicValue::Array(_))
    }
    pub fn is_object(&self) -> bool {
        matches!(self, DynamicValue::Object(_))
    }

    pub fn get(&self, key: &str) -> Option<&DynamicValue> {
        if let DynamicValue::Object(pairs) = self {
            pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_null() {
        assert!(DynamicValue::Null.is_null());
        assert!(DynamicValue::default().is_null());
    }

    #[test]
    fn accessors_return_correct_variants() {
        assert_eq!(DynamicValue::Bool(true).as_bool(), Some(true));
        assert_eq!(DynamicValue::Number(3.14).as_f64(), Some(3.14));
        assert_eq!(DynamicValue::String("hi".into()).as_str(), Some("hi"));
        let arr = DynamicValue::Array(vec![DynamicValue::Null]);
        assert!(arr.as_array().is_some());
        let obj = DynamicValue::Object(vec![("k".into(), DynamicValue::Number(1.0))]);
        assert!(obj.as_object().is_some());
    }

    #[test]
    fn get_on_object_finds_key() {
        let obj = DynamicValue::Object(vec![
            ("a".into(), DynamicValue::Number(1.0)),
            ("b".into(), DynamicValue::String("x".into())),
        ]);
        assert_eq!(obj.get("a").and_then(|v| v.as_f64()), Some(1.0));
        assert_eq!(obj.get("b").and_then(|v| v.as_str()), Some("x"));
        assert!(obj.get("c").is_none());
    }

    #[test]
    fn serialization_roundtrip_simple_variants() {
        // Null, Bool, Number, and String roundtrip cleanly through JSON.
        let cases: Vec<DynamicValue> = vec![
            DynamicValue::Null,
            DynamicValue::Bool(true),
            DynamicValue::Bool(false),
            DynamicValue::Number(42.0),
            DynamicValue::String("hello".into()),
            DynamicValue::Array(vec![
                DynamicValue::Number(1.0),
                DynamicValue::String("two".into()),
            ]),
        ];
        for val in &cases {
            let json = serde_json::to_string(val).unwrap();
            let restored: DynamicValue = serde_json::from_str(&json).unwrap();
            assert_eq!(*val, restored, "roundtrip failed for {:?}", val);
        }
    }
}
