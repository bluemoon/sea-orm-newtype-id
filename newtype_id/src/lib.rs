/// # Sea-orm Newtype ID is a ID generation system 🐚

/// - 🎲 Currently uses `nanoid`
/// - 🦓 Stripe style IDs allow for users to see what kind of IDs are being used without having to log them in the database

/// ## Rationale

/// - `String`ly typed IDs can be very error prone, think of the case where you are using more than one ID at a time
pub use sea_orm_newtype_id_domain::*;
pub use sea_orm_newtype_id_macros::def_id;

pub use smol_str;

#[cfg(test)]
mod test;
