use super::slot::Slot;
use crate::{CellTrait, StrongRefTrait, WeakRef, WeakRefTrait};
use std::ops::{Deref, DerefMut};

pub(crate) const MUT_REF_COUNT: u32 = u32::MAX;

pub struct RefMut<'t, 'u, T> {
    r: &'t mut StrongRef<'u, T>,
}

impl<'t, 'u, T> Deref for RefMut<'t, 'u, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.r.slot.get() }
    }
}

impl<'t, 'u, T> DerefMut for RefMut<'t, 'u, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.r.slot.get_mut() }
    }
}

impl<'t, 'u, T> Drop for RefMut<'t, 'u, T> {
    fn drop(&mut self) {
        self.r.slot.count.set(1); // We know there's only one strong reference at this point
    }
}

pub struct StrongRef<'t, T> {
    slot: &'t Slot<T>,
}

impl<'t, T> StrongRef<'t, T> {
    pub(crate) fn new(slot: &'t Slot<T>) -> Self {
        slot.count.add(1);
        Self { slot }
    }

    pub fn borrow_mut<'u>(&'u mut self) -> RefMut<'u, 't, T> {
        self.try_borrow_mut()
            .expect("More than one strong reference!")
    }

    pub fn try_borrow_mut<'u>(&'u mut self) -> Option<RefMut<'u, 't, T>> {
        if self.is_unique() {
            self.slot.count.set(MUT_REF_COUNT);
            Some(RefMut { r: self })
        } else {
            None
        }
    }

    pub fn remove(self) {
        let slot = self.slot;
        drop(self);
        slot.remove();
    }
}

impl<'t, T> PartialEq for StrongRef<'t, T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.slot as *const Slot<T>, other.slot as *const Slot<T>)
    }
}

impl<'t, T> Eq for StrongRef<'t, T> {}

impl<'t, T> Clone for StrongRef<'t, T> {
    fn clone(&self) -> Self {
        Self::new(self.slot)
    }
}

impl<'t, T> std::hash::Hash for StrongRef<'t, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.slot as *const Slot<T>).hash(state);
    }
}

impl<'t, T> std::fmt::Debug for StrongRef<'t, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StrongRef")
            .field("slot", &(self.slot as *const Slot<T>))
            .finish()
    }
}

impl<'t, T> Deref for StrongRef<'t, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.slot.get() }
    }
}

impl<'t, T> Drop for StrongRef<'t, T> {
    fn drop(&mut self) {
        self.slot.count.sub(1)
    }
}

impl<'t, T> TryFrom<WeakRef<'t, T>> for StrongRef<'t, T> {
    type Error = String;

    fn try_from(value: WeakRef<'t, T>) -> Result<Self, Self::Error> {
        value.strong().ok_or_else(|| "Element removed!".into())
    }
}

impl<'t, T> StrongRefTrait for StrongRef<'t, T> {
    type Weak = WeakRef<'t, T>;

    fn weak(&self) -> WeakRef<'t, T> {
        WeakRef::new(self.slot)
    }

    fn strong_count(&self) -> usize {
        self.slot.count.get() as usize
    }
}
