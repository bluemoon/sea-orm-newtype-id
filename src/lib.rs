//! ID Generation
//!
//! We use a macro to generate the appropriate structs that we need
use sea_orm::{
    sea_query::{ColumnType, ValueType, ValueTypeErr},
    Value,
};

use nanoid::nanoid;

const ID_LENGTH: u32 = 26;

/// Id generation lives here
#[derive(Debug)]
pub struct Id;

impl Id {
    /// Generates an ID with a prefix
    pub fn generate(prefix: &str) -> String {
        debug_assert!(prefix.len() <= 4);

        let id = nanoid!();
        format!("{}_{}", prefix, id)
    }
}

/// PrefixId
///
/// Is a trait that is used to add prefix based ID generation to a given domain
/// object, e.g. an object that gets inserted into the database
pub trait PrefixedId {
    /// The prefix that is desired
    const PREFIX: &'static str;

    /// Generated a new id with the given prefix
    fn new_id() -> String {
        Id::generate(Self::PREFIX)
    }

    /// Return the prefix
    fn id_prefix() -> String {
        Self::PREFIX.to_string()
    }
}

macro_rules! def_id {
    ($struct_name:ident, $prefix:literal $(| $alt_prefix:literal)* $(, { $generate_hint:tt })?) => {
      ////////////////////////////////////////////////
      // Main Struct
      ////////////////////////////////////////////////

      #[derive(Clone, Debug, Eq, PartialEq, Hash)]
      pub struct $struct_name(smol_str::SmolStr);

      impl $struct_name {
        /// Create a new ID
        pub fn new() -> Self {
          $struct_name(Self::new_id().into())
        }

        /// Extracts a string slice containing the entire id.
        #[inline(always)]
        pub fn as_str(&self) -> &str {
          self.0.as_str()
        }
      }

      impl Default for $struct_name {
         fn default() -> Self {
           Self::new()
         }
      }

      impl PartialEq<str> for $struct_name {
        fn eq(&self, other: &str) -> bool {
            self.as_str() == other
        }
      }

      impl PartialEq<&str> for $struct_name {
        fn eq(&self, other: &&str) -> bool {
            self.as_str() == *other
        }
      }

      impl PartialEq<String> for $struct_name {
        fn eq(&self, other: &String) -> bool {
            self.as_str() == other
        }
      }

      impl std::fmt::Display for $struct_name {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
          self.0.fmt(f)
        }
      }

     impl std::str::FromStr for $struct_name {
      type Err = ParseIdError;

      fn from_str(s: &str) -> Result<Self, Self::Err> {
          if !s.starts_with($prefix) $(
              && !s.starts_with($alt_prefix)
          )* {
            // N.B. For debugging
            eprintln!("bad id is: {} (expected: {:?})", s, $prefix);

            Err(ParseIdError {
                typename: stringify!($struct_name),
                expected: stringify!(id to start with $prefix $(or $alt_prefix)*),
            })
          } else {
            Ok($struct_name(s.into()))
          }
        }
      }

      impl PrefixedId for $struct_name {
        const PREFIX: &'static str = $prefix;
      }

      // TODO: make this a config
      impl serde::Serialize for $struct_name {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
          S: serde::ser::Serializer,
        {
          self.as_str().serialize(serializer)
        }
      }

      impl<'de> serde::Deserialize<'de> for $struct_name {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
          D: serde::de::Deserializer<'de>,
        {
          let s: String = serde::Deserialize::deserialize(deserializer)?;
          s.parse::<Self>().map_err(::serde::de::Error::custom)
        }
      }

      impl From<$struct_name> for Value {
        fn from(v: $struct_name) -> Self {
          Value::String(Some(Box::new(v.as_str().to_string())))
        }
      }

      impl ValueType for $struct_name {
        fn try_from(v: Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
          match v {
            Value::String(Some(x)) => {
              let v: String = *x;
              Ok(Self(v.into()))
            }
            _ => Err(ValueTypeErr),
          }
        }

        fn array_type() -> sea_orm::sea_query::ArrayType {
          sea_orm::sea_query::ArrayType::String
        }

        fn type_name() -> String {
          stringify!($type).to_owned()
        }

        fn column_type() -> ColumnType {
          ColumnType::String(Some(26))
        }
      }

      impl sea_orm::TryFromU64 for $struct_name {
        fn try_from_u64(_n: u64) -> Result<Self, sea_orm::DbErr> {
          Err(sea_orm::DbErr::ConvertFromU64(stringify!($struct_name)))

        }
      }

      impl sea_orm::TryGetable for $struct_name {
        fn try_get(
          res: &sea_orm::QueryResult,
          pre: &str,
          col: &str,
        ) -> Result<Self, sea_orm::TryGetError> {
          let val: String = res.try_get(pre, col).map_err(sea_orm::TryGetError::DbErr)?;
          Ok($struct_name(val.into()))
        }
      }

      impl sea_orm::sea_query::Nullable for $struct_name {
        fn null() -> sea_orm::Value {
          sea_orm::Value::String(None)
        }
      }

      impl sea_orm::IntoActiveValue<Self> for $struct_name {
        fn into_active_value(self) -> sea_orm::ActiveValue<Self> {
          sea_orm::Set(self)
        }
      }

      // TODO: make this a config
      #[async_graphql::Scalar]
      impl async_graphql::ScalarType for $struct_name {
        fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
          if let async_graphql::Value::String(value) = &value {
            Ok($struct_name(value.into()))
          } else {
            Err(async_graphql::InputValueError::expected_type(value))
          }
        }

        fn to_value(&self) -> async_graphql::Value {
          async_graphql::Value::String(self.to_string())
        }
      }

    };
  }

#[derive(Clone, Debug)]
pub struct ParseIdError {
    typename: &'static str,
    expected: &'static str,
}

impl std::fmt::Display for ParseIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid `{}`, expected {}", self.typename, self.expected)
    }
}

impl std::error::Error for ParseIdError {
    fn description(&self) -> &str {
        "error parsing an id"
    }
}
