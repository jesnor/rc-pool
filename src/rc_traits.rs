use std::{
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
    slice::Iter,
};

pub trait StrongRefTrait: Deref {
    type Weak: WeakRefTrait<Target = Self::Target>;

    type RefMut<'t>: DerefMut<Target = Self::Target>
    where
        Self: 't;

    #[must_use]
    fn weak(&self) -> Self::Weak;

    #[must_use]
    fn strong_count(&self) -> usize;

    #[must_use]
    fn is_unique(&self) -> bool {
        self.strong_count() == 1
    }

    #[must_use]
    fn get_mut(&mut self) -> Option<Self::RefMut<'_>>;
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
    type RefMut<'t> = &'t mut T where Self: 't;

    fn weak(&self) -> Self::Weak {
        Rc::downgrade(self)
    }

    fn strong_count(&self) -> usize {
        Rc::strong_count(self)
    }

    #[must_use]
    fn get_mut(&mut self) -> Option<&mut Self::Target> {
        Rc::get_mut(self)
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

pub trait WeakSliceExt<T> {
    fn iter_strong(&self) -> StrongIterator<T>;
}

impl<V: AsRef<[T]>, T> WeakSliceExt<T> for V {
    fn iter_strong(&self) -> StrongIterator<T> {
        StrongIterator {
            iter: self.as_ref().iter(),
        }
    }
}

pub struct StrongIterator<'t, T> {
    iter: Iter<'t, T>,
}

impl<'t, T: WeakRefTrait> Iterator for StrongIterator<'t, T> {
    type Item = T::Strong;

    fn next(&mut self) -> Option<Self::Item> {
        for r in self.iter.by_ref() {
            if let Some(sr) = r.strong() {
                return Some(sr);
            }
        }

        None
    }
}
