use super::{Count, Index, Version};
use crate::CellTrait;
use std::{
    cell::{Cell, UnsafeCell},
    mem::MaybeUninit,
};

pub struct Slot<T> {
    pub(crate) value: UnsafeCell<MaybeUninit<T>>,
    pub(crate) version: Cell<Version>,
    pub(crate) count: Cell<Count>,
    pub(crate) index: Cell<Index>,
}

impl<T> Slot<T> {
    #[must_use]
    pub(crate) unsafe fn get(&self) -> &T {
        debug_assert!(self.count.get() > 0);
        (*self.value.get()).assume_init_ref()
    }

    #[must_use]
    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn get_mut(&self) -> &mut T {
        (*self.value.get()).assume_init_mut()
    }

    pub(crate) unsafe fn set_value(&self, value: T) {
        debug_assert!(self.is_free());
        debug_assert!(self.count.get() == 0);
        (*self.value.get()).write(value);
        self.incr_version();
    }

    unsafe fn drop_value(&self) {
        debug_assert!(!self.is_free());
        debug_assert!(self.count.get() == 0);
        (*self.value.get()).assume_init_drop();
        self.incr_version();
    }

    pub(crate) fn is_free(&self) -> bool {
        self.version.get() & 1 == 0
    }

    fn incr_version(&self) {
        self.version.set(self.version.get().wrapping_add(1));
    }

    unsafe fn head(&self) -> &Slot<T> {
        unsafe { &*(self as *const Slot<T>).offset(-(self.index.get() as isize)) }
    }

    pub(crate) fn remove(&self) {
        assert_eq!(
            self.count.get(),
            0,
            "Can't remove item with strong references!"
        );

        unsafe { self.drop_value() };
        let index = self.index.get();
        let head = unsafe { self.head() };
        self.index.set(head.index.get());
        head.index.set(index);
        head.count.sub(1);
    }
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Self {
            value: UnsafeCell::new(MaybeUninit::uninit()),
            version: 0.into(),
            count: 0.into(),
            index: 0.into(),
        }
    }
}
