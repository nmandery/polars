use serde::{Deserialize, Serialize};
pub mod chunked_array;
pub mod series;
use crate::prelude::*;

// Serde calls this the definition of the remote type. It is just a copy of the
// remote data structure. The `remote` attribute gives the path to the actual
// type we intend to derive code for.
#[derive(Serialize, Deserialize, Debug)]
#[serde(remote = "TimeUnit")]
enum TimeUnitDef {
    /// Time in seconds.
    Second,
    /// Time in milliseconds.
    Millisecond,
    /// Time in microseconds.
    Microsecond,
    /// Time in nanoseconds.
    Nanosecond,
}

/// Intermediate enum. Needed because [crate::datatypes::DataType] has
/// a &static str and thus requires Deserialize<&static>
#[derive(Serialize, Deserialize, Debug)]
enum DeDataType<'a> {
    Boolean,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Int8,
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
    Utf8,
    Date,
    Datetime,
    #[serde(with = "TimeUnitDef")]
    Time64(TimeUnit),
    List,
    Object(&'a str),
    Null,
    Categorical,
}

impl From<&DataType> for DeDataType<'_> {
    fn from(dt: &DataType) -> Self {
        match dt {
            DataType::Int32 => DeDataType::Int32,
            DataType::UInt32 => DeDataType::UInt32,
            DataType::Int64 => DeDataType::Int64,
            DataType::UInt64 => DeDataType::UInt64,
            DataType::Date => DeDataType::Date,
            DataType::Datetime => DeDataType::Datetime,
            DataType::Float32 => DeDataType::Float32,
            DataType::Float64 => DeDataType::Float64,
            DataType::Utf8 => DeDataType::Utf8,
            DataType::Boolean => DeDataType::Boolean,
            DataType::Null => DeDataType::Null,
            DataType::List(_) => DeDataType::List,
            #[cfg(feature = "object")]
            DataType::Object(s) => DeDataType::Object(s),
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_serde() -> Result<()> {
        let ca = UInt32Chunked::new_from_opt_slice("foo", &[Some(1), None, Some(2)]);

        let json = serde_json::to_string(&ca).unwrap();
        dbg!(&json);

        let out = serde_json::from_str::<Series>(&json).unwrap();
        assert!(ca.into_series().series_equal_missing(&out));

        let ca = Utf8Chunked::new_from_opt_slice("foo", &[Some("foo"), None, Some("bar")]);

        let json = serde_json::to_string(&ca).unwrap();
        dbg!(&json);

        let out = serde_json::from_str::<Series>(&json).unwrap();
        assert!(ca.into_series().series_equal_missing(&out));

        Ok(())
    }

    #[test]
    fn test_serde_bincode() -> Result<()> {
        let ca = UInt32Chunked::new_from_opt_slice("foo", &[Some(1), None, Some(2)]);

        let bytes = bincode::serialize(&ca).unwrap();

        let out = bincode::deserialize::<Series>(&bytes).unwrap();
        assert!(ca.into_series().series_equal_missing(&out));

        let ca = Utf8Chunked::new_from_opt_slice("foo", &[Some("foo"), None, Some("bar")]);

        let bytes = bincode::serialize(&ca).unwrap();

        let out = bincode::deserialize::<Series>(&bytes).unwrap();
        assert!(ca.into_series().series_equal_missing(&out));

        Ok(())
    }

    #[test]
    fn test_serde_df() {
        let s = Series::new("foo", &[1, 2, 3]);
        let s1 = Series::new("bar", &[Some(true), None, Some(false)]);
        let s_list = Series::new("list", &[s.clone(), s.clone(), s.clone()]);

        let df = DataFrame::new(vec![s, s_list, s1]).unwrap();
        let json = serde_json::to_string(&df).unwrap();
        dbg!(&json);
        let out = serde_json::from_str::<DataFrame>(&json).unwrap();
        assert!(df.frame_equal_missing(&out));
    }

    #[test]
    fn test_serde_df_bincode() {
        let s = Series::new("foo", &[1, 2, 3]);
        let s1 = Series::new("bar", &[Some(true), None, Some(false)]);
        let s_list = Series::new("list", &[s.clone(), s.clone(), s.clone()]);

        let df = DataFrame::new(vec![s, s_list, s1]).unwrap();
        let bytes = bincode::serialize(&df).unwrap();

        let out = bincode::deserialize::<DataFrame>(&bytes).unwrap();
        assert!(df.frame_equal_missing(&out));
    }
}
