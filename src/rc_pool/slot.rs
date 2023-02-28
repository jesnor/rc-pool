use super::{Count, Index, Version};
use crate::CellTrait;
use std::{
    cell::{Cell, UnsafeCell},
    mem::MaybeUninit,
    num::NonZeroUsize,
};

pub struct Slot<T> {
    pub(crate) item: UnsafeCell<MaybeUninit<T>>,
    pub(crate) version: Cell<Version>,
    pub(crate) count: Cell<Count>,
    pub(crate) index: Cell<Index>,
}

impl<T> Slot<T> {
    #[must_use]
    pub(crate) unsafe fn get(&self) -> &T {
        debug_assert!(self.count.get() > 0);
        (*self.item.get()).assume_init_ref()
    }

    #[must_use]
    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn get_mut(&self) -> &mut T {
        (*self.item.get()).assume_init_mut()
    }

    pub(crate) unsafe fn set_value(&self, value: T) {
        debug_assert!(self.is_free());
        debug_assert!(self.count.get() == 0);
        (*self.item.get()).write(value);
        self.incr_version();
    }

    pub(crate) fn is_free(&self) -> bool {
        self.version.get().get() & 1 == 1
    }

    fn incr_version(&self) {
        self.version
            .set(unsafe { NonZeroUsize::new_unchecked(self.version.get().get() + 1) });
    }

    unsafe fn head(&self) -> &Slot<T> {
        unsafe { &*(self as *const Slot<T>).offset(-(self.index.get() as isize)) }
    }

    pub(crate) fn take_item(&self) -> T {
        debug_assert!(!self.is_free());

        debug_assert!(
            self.count.get() == 0,
            "Can't take item with strong references!"
        );

        self.incr_version();
        let index = self.index.get();
        let head = unsafe { self.head() };
        self.index.set(head.index.get());
        head.index.set(index);
        head.count.sub(1);
        unsafe { (*self.item.get()).assume_init_read() }
    }
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Self {
            item: UnsafeCell::new(MaybeUninit::uninit()),
            version: unsafe { NonZeroUsize::new_unchecked(1).into() },
            count: 0.into(),
            index: 0.into(),
        }
    }
}
