use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum JsonTernary<T> {
    Value(T), // Set the aggregate value to the value of T
    Null,     // Set the aggregate value to null
    Absent    // Do not change the aggregate value
}

impl<T> JsonTernary<T> {
    pub const fn is_value(&self) -> bool {
        matches!(*self, JsonTernary::Value(_))
    }
    pub const fn is_null(&self) -> bool {
        matches!(*self, JsonTernary::Null)
    }
    pub const fn is_absent(&self) -> bool {
        matches!(*self, JsonTernary::Absent)
    }
}

// https://stackoverflow.com/a/44332837
impl<T> Default for JsonTernary<T> {
    fn default() -> Self {
        JsonTernary::Absent
    }
}

impl<T> From<Option<T>> for JsonTernary<T> {
    fn from(opt: Option<T>) -> JsonTernary<T> {
        match opt {
            Some(v) => JsonTernary::Value(v),
            None => JsonTernary::Null,
        }
    }
}

impl<'de, T> Deserialize<'de> for JsonTernary<T> where T: Deserialize<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Option::deserialize(deserializer).map(Into::into)
    }
}

// See https://serde.rs/impl-serialize.html
impl<T> Serialize for JsonTernary<T> where T: Serialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            JsonTernary::Value(t) => serializer.serialize_some(t),
            JsonTernary::Null => serializer.serialize_none(),
            JsonTernary::Absent => serializer.serialize_none(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::json_ternary::JsonTernary;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Record {
        #[serde(default)]
        #[serde(skip_serializing_if = "JsonTernary::is_absent")]
        a: JsonTernary<String>,

        #[serde(default)]
        #[serde(skip_serializing_if = "JsonTernary::is_absent")]
        b: JsonTernary<u32>,

        #[serde(default)]
        #[serde(skip_serializing_if = "JsonTernary::is_absent")]
        c: JsonTernary<Vec<i32>>
    }

    #[test]
    pub fn test_ternary_value() {
        let t = JsonTernary::Value(String::from("123"));
        assert!(t.is_value());
        assert!(!t.is_null());
        assert!(!t.is_absent());

        let json = serde_json::to_string(&t);
        assert!(json.is_ok());
        assert_eq!(json.unwrap(), String::from(r#""123""#));
    }

    #[test]
    pub fn test_ternary_null() {
        let t: JsonTernary<u32> = JsonTernary::Null;
        assert!(!t.is_value());
        assert!(t.is_null());
        assert!(!t.is_absent());

        let json = serde_json::to_string(&t);
        assert!(json.is_ok());
        assert_eq!(json.unwrap(), String::from("null"));
    }

    #[test]
    pub fn test_ternary_absent() {
        let t: JsonTernary<u32> = JsonTernary::Absent;
        assert!(!t.is_value());
        assert!(!t.is_null());
        assert!(t.is_absent());

        let json = serde_json::to_string(&t);
        assert!(json.is_ok());
        assert_eq!(json.unwrap(), String::from("null"));
    }

    #[test]
    pub fn test_serde_record_value() {
        let record_ref = Record{
            a: JsonTernary::Value(String::from("Foo")),
            b: JsonTernary::Value(123),
            c: JsonTernary::Value(vec![3, -5, 7])
        };
        let json_ref = r#"{"a":"Foo","b":123,"c":[3,-5,7]}"#;
        serde_and_verify(&record_ref, json_ref);
    }

    #[test]
    pub fn test_serialize_record_null() {
        let record_ref = Record{
            a: JsonTernary::Null,
            b: JsonTernary::Null,
            c: JsonTernary::Null
        };
        let json_ref = r#"{"a":null,"b":null,"c":null}"#;
        serde_and_verify(&record_ref, json_ref);
    }

    #[test]
    pub fn test_serialize_record_absent() {
        let record_ref = Record{
            a: JsonTernary::Absent,
            b: JsonTernary::Absent,
            c: JsonTernary::Absent
        };
        let json_ref = r#"{}"#;
        serde_and_verify(&record_ref, json_ref);
    }

    fn serde_and_verify(record_ref: &Record, json_ref: &str) {
        let json = serde_json::to_string(&record_ref);
        assert!(json.is_ok());
        assert_eq!(json.unwrap(), String::from(json_ref));

        let record : Result<Record, serde_json::Error> = serde_json::from_str(json_ref);
        assert!(record.is_ok());
        assert_eq!(record.unwrap(), *record_ref);
    }
}