use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

pub trait Wordlike:
    BitAnd<Output = Self>
    + BitAndAssign
    + BitOr<Output = Self>
    + BitOrAssign
    + BitXor<Output = Self>
    + BitXorAssign
    + Copy
    + Eq
    + Not<Output = Self>
    + Ord
{
    const ZERO: Self;
    fn count_ones(this: Self) -> u32;
    fn incr(self) -> Self;
}

macro_rules! impl_word {
    ($n: ty) => {
        impl Wordlike for $n {
            const ZERO: Self = 0;
            #[inline]
            fn count_ones(this: Self) -> u32 {
                this.count_ones()
            }
            #[inline]
            fn incr(self) -> Self {
                self + 1
            }
        }
    };
}

impl_word!(u8);
impl_word!(u16);
impl_word!(u32);
impl_word!(u64);
impl_word!(u128);
impl_word!(usize);
