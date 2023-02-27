use crate::StrongRefTrait;

pub trait Pool {
    type Item;

    type Ref<'t>: StrongRefTrait<Target = Self::Item>
    where
        Self: 't;

    #[must_use]
    fn insert(&self, value: Self::Item) -> Self::Ref<'_>;
}
