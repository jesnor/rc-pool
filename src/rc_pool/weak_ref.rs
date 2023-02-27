use super::slot::Slot;
use crate::{StrongRef, StrongRefTrait, Version, WeakRefTrait, MUT_REF_COUNT};

pub struct WeakRef<'t, T> {
    slot: &'t Slot<T>,
    version: Version,
}

impl<'t, T> WeakRef<'t, T> {
    #[must_use]
    pub(crate) fn new(slot: &'t Slot<T>) -> Self {
        Self {
            slot,
            version: slot.version.get(),
        }
    }

    pub fn remove(&self) -> bool {
        if self.is_valid() {
            self.slot.remove();
            true
        } else {
            false
        }
    }
}

impl<'t, T> WeakRefTrait for WeakRef<'t, T> {
    type Target = T;
    type Strong = StrongRef<'t, T>;

    #[must_use]
    fn strong(&self) -> Option<StrongRef<'t, T>> {
        if self.is_valid() {
            assert_ne!(
                self.slot.count.get(),
                MUT_REF_COUNT,
                "Already borrowed as mutable!"
            );

            Some(StrongRef::new(self.slot))
        } else {
            None
        }
    }

    #[must_use]
    fn is_valid(&self) -> bool {
        self.version == self.slot.version.get()
    }
}

impl<'t, T> From<StrongRef<'t, T>> for WeakRef<'t, T> {
    fn from(r: StrongRef<'t, T>) -> Self {
        r.weak()
    }
}

impl<'t, T> PartialEq for WeakRef<'t, T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.slot as *const Slot<T>, other.slot as *const Slot<T>)
            && self.version == other.version
    }
}

impl<'t, T> Eq for WeakRef<'t, T> {}

impl<'t, T> Clone for WeakRef<'t, T> {
    #[must_use]
    fn clone(&self) -> Self {
        Self {
            slot: self.slot,
            version: self.version,
        }
    }
}

impl<'t, T> Copy for WeakRef<'t, T> {}

impl<'t, T> std::hash::Hash for WeakRef<'t, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.slot as *const Slot<T>).hash(state);
        self.version.hash(state);
    }
}

impl<'t, T> std::fmt::Debug for WeakRef<'t, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakRef")
            .field("slot", &(self.slot as *const Slot<T>))
            .field("version", &self.version)
            .finish()
    }
}
