//! ID Generation
//!
//! We use a macro to generate the appropriate structs that we need
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, ExprLit, Ident, Token};

extern crate proc_macro;

struct DefId {
    struct_name: Ident,
    prefix: ExprLit,
}

impl Parse for DefId {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let struct_name: Ident = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let prefix: ExprLit = input.parse()?;

        Ok(DefId {
            struct_name,
            prefix,
        })
    }
}

#[proc_macro]
pub fn def_id(tokens: TokenStream) -> TokenStream {
    let DefId {
        struct_name,
        prefix,
    } = syn::parse_macro_input!(tokens as DefId);

    let async_graphql_impl =
        FeatureAsyncGraphQL::new(cfg!(with_async_graphql), struct_name.clone());
    let serde_impl = FeatureSerde::new(cfg!(with_serde), struct_name.clone());

    quote! {
      ////////////////////////////////////////////////
      // Main Struct
      ////////////////////////////////////////////////

      #[derive(Clone, Debug, Eq, PartialEq, Hash)]
      pub struct #struct_name(sea_orm_newtype_id::smol_str::SmolStr);

      impl #struct_name {
        /// Create a new ID
        pub fn new() -> Self {
          #struct_name(<Self as sea_orm_newtype_id::PrefixedId>::new_id().into())
        }

        /// Extracts a string slice containing the entire id.
        #[inline(always)]
        pub fn as_str(&self) -> &str {
          self.0.as_str()
        }
      }

      impl Default for #struct_name {
         fn default() -> Self {
           Self::new()
         }
      }

      impl PartialEq<str> for #struct_name {
        fn eq(&self, other: &str) -> bool {
            self.as_str() == other
        }
      }

      impl PartialEq<&str> for #struct_name {
        fn eq(&self, other: &&str) -> bool {
            self.as_str() == *other
        }
      }

      impl PartialEq<String> for #struct_name {
        fn eq(&self, other: &String) -> bool {
            self.as_str() == other
        }
      }

      impl std::fmt::Display for #struct_name {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
          self.0.fmt(f)
        }
      }

     impl std::str::FromStr for #struct_name {
      type Err = sea_orm_newtype_id::ParseIdError;

      fn from_str(s: &str) -> Result<Self, Self::Err> {
          if !s.starts_with(#prefix) {
            // N.B. For debugging
            eprintln!("bad id is: {} (expected: {:?})", s, #prefix);

            Err(sea_orm_newtype_id::ParseIdError {
                typename: stringify!(#struct_name),
                expected: stringify!(id to start with #prefix)
            })
          } else {
            Ok(#struct_name(s.into()))
          }
        }
      }

      impl sea_orm_newtype_id::PrefixedId for #struct_name {
        const PREFIX: &'static str = #prefix;
      }

      impl From<#struct_name> for sea_orm::Value {
        fn from(v: #struct_name) -> Self {
          sea_orm::Value::String(Some(Box::new(v.as_str().to_string())))
        }
      }

      impl sea_orm::sea_query::ValueType for #struct_name {
        fn try_from(v: sea_orm::Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
          match v {
            sea_orm::Value::String(Some(x)) => {
              let v: String = *x;
              Ok(Self(v.into()))
            }
            _ => Err(sea_orm::sea_query::ValueTypeErr),
          }
        }

        fn array_type() -> sea_orm::sea_query::ArrayType {
          sea_orm::sea_query::ArrayType::String
        }

        fn type_name() -> String {
          stringify!($type).to_owned()
        }

        fn column_type() -> sea_orm::sea_query::ColumnType {
          sea_orm::sea_query::ColumnType::String(Some(26))
        }
      }

      impl sea_orm::TryFromU64 for #struct_name {
        fn try_from_u64(_n: u64) -> Result<Self, sea_orm::error::DbErr> {
          Err(sea_orm::error::DbErr::ConvertFromU64(stringify!(#struct_name)))
        }
      }

      impl sea_orm::TryGetable for #struct_name {
        fn try_get(
          res: &sea_orm::QueryResult,
          pre: &str,
          col: &str,
        ) -> Result<Self, sea_orm::TryGetError> {
          let val: String = res.try_get(pre, col).map_err(sea_orm::TryGetError::DbErr)?;
          Ok(#struct_name(val.into()))
        }
      }

      impl sea_orm::sea_query::Nullable for #struct_name {
        fn null() -> sea_orm::Value {
          sea_orm::Value::String(None)
        }
      }

      impl sea_orm::IntoActiveValue<Self> for #struct_name {
        fn into_active_value(self) -> sea_orm::ActiveValue<Self> {
          sea_orm::Set(self)
        }
      }

      #async_graphql_impl
      #serde_impl
    }
    .into()
}

struct FeatureSerde {
    enabled: bool,
    struct_name: Ident,
}

impl FeatureSerde {
    fn new(enabled: bool, struct_name: Ident) -> Self {
        Self {
            enabled,
            struct_name,
        }
    }
}

impl ToTokens for FeatureSerde {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_name = self.struct_name.clone();
        if self.enabled {
            tokens.extend(quote! {
                impl serde::Serialize for #struct_name {
                  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                  where
                    S: serde::ser::Serializer,
                  {
                    self.as_str().serialize(serializer)
                  }
                }

                impl<'de> serde::Deserialize<'de> for #struct_name {
                  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                  where
                    D: serde::de::Deserializer<'de>,
                  {
                    let s: String = serde::Deserialize::deserialize(deserializer)?;
                    s.parse::<Self>().map_err(::serde::de::Error::custom)
                  }
                }
            });
        }
    }
}

struct FeatureAsyncGraphQL {
    enabled: bool,
    struct_name: Ident,
}

impl FeatureAsyncGraphQL {
    fn new(enabled: bool, struct_name: Ident) -> Self {
        Self {
            enabled,
            struct_name,
        }
    }
}

impl ToTokens for FeatureAsyncGraphQL {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_name = self.struct_name.clone();

        if self.enabled {
            tokens.extend(quote! {
             #[async_graphql::Scalar]
             impl async_graphql::ScalarType for #struct_name {
               fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
                 if let async_graphql::Value::String(value) = &value {
                   Ok(#struct_name(value.into()))
                 } else {
                   Err(async_graphql::InputValueError::expected_type(value))
                 }
               }

               fn to_value(&self) -> async_graphql::Value {
                 async_graphql::Value::String(self.to_string())
               }
             }
            })
        }
    }
}
