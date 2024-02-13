use crate::{Enum, Wordlike};

pub trait OptionableEnum: Enum {
    type RepForOptional;
}

impl<T: OptionableEnum> Enum for Option<T>
where
    T::RepForOptional: Wordlike + From<T::Rep>,
{
    type Rep = T::RepForOptional;

    const SIZE: usize = T::SIZE + 1;

    const MIN: Self = None;

    const MAX: Self = Some(T::MAX);

    fn succ(self) -> Option<Self> {
        match self {
            None => Some(Some(T::MIN)),
            Some(e) => e.succ().map(Some),
        }
    }

    fn pred(self) -> Option<Self> {
        self.map(T::pred)
    }

    fn bit(self) -> Self::Rep {
        match self {
            None => Self::Rep::from(T::MIN.bit()),
            Some(e) => Self::Rep::from(e.bit()).incr(),
        }
        .into()
    }

    fn index(self) -> usize {
        match self {
            None => 0,
            Some(e) => e.index() + 1,
        }
    }

    fn from_index(i: usize) -> Option<Self> {
        if i == 0 {
            Some(None)
        } else {
            T::from_index(i - 1).map(Some)
        }
    }
}

impl OptionableEnum for bool {
    type RepForOptional = u8;
}
