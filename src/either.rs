#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    pub fn left(&self) -> Option<&L> {
        match self {
            Either::Left(v) => Some(v),
            Either::Right(_) => None,
        }
    }

    pub fn right(&self) -> Option<&R> {
        match self {
            Either::Left(_) => None,
            Either::Right(r) => Some(r),
        }
    }

    pub fn take_left(self) -> Option<L> {
        match self {
            Either::Left(v) => Some(v),
            Either::Right(_) => None,
        }
    }

    pub fn take_right(self) -> Option<R> {
        match self {
            Either::Left(_) => None,
            Either::Right(v) => Some(v),
        }
    }
}
