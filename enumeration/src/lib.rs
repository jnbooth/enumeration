#[cfg(test)]
#[macro_use]
extern crate enumeration_derive;

#[macro_use]
mod enum_trait;
pub use enum_trait::Enum;
mod set;
pub use set::{EnumSet, __private};
mod map;
pub use map::EnumMap;
mod wordlike;
pub use wordlike::Wordlike;

mod external_trait_impls;
