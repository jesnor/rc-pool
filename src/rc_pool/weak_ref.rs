use super::slot::Slot;
use crate::{StrongRef, StrongRefTrait, Version, WeakRefTrait, MUT_REF_COUNT};

pub struct WeakRef<'t, T, const MANUAL_DROP: bool> {
    slot: &'t Slot<T>,
    version: Version,
}

impl<'t, T, const MANUAL_DROP: bool> WeakRef<'t, T, MANUAL_DROP> {
    #[must_use]
    pub(crate) fn new(slot: &'t Slot<T>) -> Self {
        Self {
            slot,
            version: slot.version.get(),
        }
    }

    #[must_use]
    pub fn try_take_item(&self) -> Option<T> {
        if self.is_valid() && self.slot.count.get() == 0 {
            Some(self.slot.take_item())
        } else {
            None
        }
    }

    #[must_use]
    pub fn take_item(&self) -> T {
        self.try_take_item()
            .expect("Can't take item with references!")
    }

    #[must_use]
    pub fn try_drop_item(&self) -> bool {
        if self.is_valid() && self.slot.count.get() == 0 {
            self.slot.take_item();
            true
        } else {
            false
        }
    }

    pub fn drop_item(&self) {
        assert!(self.try_drop_item(), "Can't drop item with references!");
    }
}

impl<'t, T, const MANUAL_DROP: bool> WeakRefTrait for WeakRef<'t, T, MANUAL_DROP> {
    type Target = T;
    type Strong = StrongRef<'t, T, MANUAL_DROP>;

    #[must_use]
    fn strong(&self) -> Option<Self::Strong> {
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

impl<'t, T, const MANUAL_DROP: bool> From<StrongRef<'t, T, MANUAL_DROP>>
    for WeakRef<'t, T, MANUAL_DROP>
{
    fn from(r: StrongRef<'t, T, MANUAL_DROP>) -> Self {
        r.weak()
    }
}

impl<'t, T, const MANUAL_DROP: bool> PartialEq for WeakRef<'t, T, MANUAL_DROP> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.slot as *const Slot<T>, other.slot as *const Slot<T>)
            && self.version == other.version
    }
}

impl<'t, T, const MANUAL_DROP: bool> Eq for WeakRef<'t, T, MANUAL_DROP> {}

impl<'t, T, const MANUAL_DROP: bool> Clone for WeakRef<'t, T, MANUAL_DROP> {
    #[must_use]
    fn clone(&self) -> Self {
        Self {
            slot: self.slot,
            version: self.version,
        }
    }
}

impl<'t, T, const MANUAL_DROP: bool> Copy for WeakRef<'t, T, MANUAL_DROP> {}

impl<'t, T, const MANUAL_DROP: bool> std::hash::Hash for WeakRef<'t, T, MANUAL_DROP> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.slot as *const Slot<T>).hash(state);
        self.version.hash(state);
    }
}

impl<'t, T, const MANUAL_DROP: bool> std::fmt::Debug for WeakRef<'t, T, MANUAL_DROP> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakRef")
            .field("slot", &(self.slot as *const Slot<T>))
            .field("version", &self.version)
            .finish()
    }
}
