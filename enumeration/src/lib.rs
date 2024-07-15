#[cfg(not(test))]
#[cfg(feature = "enumeration_derive")]
extern crate enumeration_derive;

#[cfg(test)]
#[cfg(feature = "enumeration_derive")]
#[macro_use]
extern crate enumeration_derive;

#[cfg(feature = "enumeration_derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub use enumeration_derive::Enum;

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
