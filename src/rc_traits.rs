use std::{
    ops::Deref,
    rc::{Rc, Weak},
};

pub trait StrongRefTrait: Deref {
    type Weak: WeakRefTrait<Target = Self::Target>;

    #[must_use]
    fn weak(&self) -> Self::Weak;

    #[must_use]
    fn strong_count(&self) -> usize;

    #[must_use]
    fn is_unique(&self) -> bool {
        self.strong_count() == 1
    }
}

pub trait WeakRefTrait {
    type Target;
    type Strong: StrongRefTrait<Target = Self::Target>;

    #[must_use]
    fn strong(&self) -> Option<Self::Strong>;

    #[must_use]
    fn is_valid(&self) -> bool {
        self.strong().is_some()
    }
}

impl<T> StrongRefTrait for Rc<T> {
    type Weak = Weak<T>;

    fn weak(&self) -> Self::Weak {
        Rc::downgrade(self)
    }

    fn strong_count(&self) -> usize {
        Rc::strong_count(self)
    }
}

impl<T> WeakRefTrait for Weak<T> {
    type Target = T;
    type Strong = Rc<T>;

    fn strong(&self) -> Option<Self::Strong> {
        self.upgrade()
    }

    fn is_valid(&self) -> bool {
        self.strong_count() > 0
    }
}
