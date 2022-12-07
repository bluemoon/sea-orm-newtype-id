use nanoid::nanoid;

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

/// Error for parsing
#[derive(Clone, Debug)]
pub struct ParseIdError {
    pub typename: &'static str,
    pub expected: &'static str,
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
