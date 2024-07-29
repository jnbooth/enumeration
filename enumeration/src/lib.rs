#![allow(clippy::manual_map)]

#[cfg(not(test))]
#[cfg(feature = "enumeration_derive")]
extern crate enumeration_derive;

#[cfg(test)]
#[cfg(feature = "enumeration_derive")]
#[macro_use]
#[cfg(feature = "derive")]
extern crate enumeration_derive;

#[cfg(feature = "enumeration_derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub use enumeration_derive::Enum;

#[macro_use]
mod enumerate;
pub use enumerate::{Enum, Enumeration};
pub mod set;
pub use set::{EnumSet, __private};

pub mod map;
pub use map::{Entry, EnumMap, OccupiedEntry, VacantEntry};

mod wordlike;
pub use wordlike::Wordlike;

mod external_trait_impls;
