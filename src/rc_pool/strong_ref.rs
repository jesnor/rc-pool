use super::slot::Slot;
use crate::{CellTrait, Either, StrongRefTrait, WeakRef, WeakRefTrait};
use std::ops::{Deref, DerefMut};

pub(crate) const MUT_REF_COUNT: u32 = u32::MAX;

pub struct RefMut<'t, 'u, T, const MANUAL_DROP: bool> {
    r: &'t mut StrongRef<'u, T, MANUAL_DROP>,
}

impl<'t, 'u, T, const MANUAL_DROP: bool> Deref for RefMut<'t, 'u, T, MANUAL_DROP> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.r.slot.get() }
    }
}

impl<'t, 'u, T, const MANUAL_DROP: bool> DerefMut for RefMut<'t, 'u, T, MANUAL_DROP> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.r.slot.get_mut() }
    }
}

impl<'t, 'u, T, const MANUAL_DROP: bool> Drop for RefMut<'t, 'u, T, MANUAL_DROP> {
    fn drop(&mut self) {
        self.r.slot.count.set(1); // We know there's only one strong reference at this point
    }
}

pub struct StrongRef<'t, T, const MANUAL_DROP: bool> {
    slot: &'t Slot<T>,
}

impl<'t, T, const MANUAL_DROP: bool> StrongRef<'t, T, MANUAL_DROP> {
    #[must_use]
    pub(crate) fn new(slot: &'t Slot<T>) -> Self {
        slot.count.add(1);
        Self { slot }
    }

    #[must_use]
    pub fn get_mut<'u>(&'u mut self) -> RefMut<'u, 't, T, MANUAL_DROP> {
        self.try_get_mut().expect("More than one strong reference!")
    }

    #[must_use]
    pub fn try_get_mut<'u>(&'u mut self) -> Option<RefMut<'u, 't, T, MANUAL_DROP>> {
        if self.is_unique() {
            self.slot.count.set(MUT_REF_COUNT);
            Some(RefMut { r: self })
        } else {
            None
        }
    }

    #[must_use]
    pub fn take_item(self) -> T {
        match self.try_take_item() {
            Either::Left(v) => v,
            Either::Right(_) => panic!("Can't take item with strong references!"),
        }
    }

    #[must_use]
    pub fn try_take_item(self) -> Either<T, Self> {
        if self.is_unique() {
            let slot = self.slot;
            drop(self);
            Either::Left(slot.take_item())
        } else {
            Either::Right(self)
        }
    }

    pub fn drop_item(self) {
        assert!(
            self.try_drop_item().is_none(),
            "Can't take item with strong references!"
        );
    }

    #[must_use]
    pub fn try_drop_item(self) -> Option<Self> {
        match self.try_take_item() {
            Either::Left(_) => None,
            Either::Right(s) => Some(s),
        }
    }
}

impl<'t, T, const MANUAL_DROP: bool> PartialEq for StrongRef<'t, T, MANUAL_DROP> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.slot as *const Slot<T>, other.slot as *const Slot<T>)
    }
}

impl<'t, T, const MANUAL_DROP: bool> Eq for StrongRef<'t, T, MANUAL_DROP> {}

impl<'t, T, const MANUAL_DROP: bool> Clone for StrongRef<'t, T, MANUAL_DROP> {
    fn clone(&self) -> Self {
        Self::new(self.slot)
    }
}

impl<'t, T, const MANUAL_DROP: bool> std::hash::Hash for StrongRef<'t, T, MANUAL_DROP> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.slot as *const Slot<T>).hash(state);
    }
}

impl<'t, T, const MANUAL_DROP: bool> std::fmt::Debug for StrongRef<'t, T, MANUAL_DROP> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StrongRef")
            .field("slot", &(self.slot as *const Slot<T>))
            .finish()
    }
}

impl<'t, T, const MANUAL_DROP: bool> Deref for StrongRef<'t, T, MANUAL_DROP> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.slot.get() }
    }
}

impl<'t, T, const MANUAL_DROP: bool> Drop for StrongRef<'t, T, MANUAL_DROP> {
    fn drop(&mut self) {
        self.slot.count.sub(1);

        if !MANUAL_DROP && self.slot.count.get() == 0 {
            self.slot.take_item();
        }
    }
}

impl<'t, T, const MANUAL_DROP: bool> TryFrom<WeakRef<'t, T, MANUAL_DROP>>
    for StrongRef<'t, T, MANUAL_DROP>
{
    type Error = String;

    fn try_from(value: WeakRef<'t, T, MANUAL_DROP>) -> Result<Self, Self::Error> {
        value.strong().ok_or_else(|| "Element removed!".into())
    }
}

impl<'t, T, const MANUAL_DROP: bool> StrongRefTrait for StrongRef<'t, T, MANUAL_DROP> {
    type Weak = WeakRef<'t, T, MANUAL_DROP>;

    type RefMut<'u> = RefMut<'u, 't, T, MANUAL_DROP> where Self: 'u;

    fn weak(&self) -> Self::Weak {
        WeakRef::new(self.slot)
    }

    fn strong_count(&self) -> usize {
        self.slot.count.get() as usize
    }

    fn get_mut(&mut self) -> Option<Self::RefMut<'_>> {
        self.try_get_mut()
    }
}
