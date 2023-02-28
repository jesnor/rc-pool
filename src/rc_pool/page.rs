use super::slot::Slot;
use crate::{CellTrait, Index, PoolHeader};
use std::{
    cell::Cell,
    mem::{ManuallyDrop, MaybeUninit},
    num::NonZeroUsize,
    ops::Deref,
};

pub(crate) struct PageHeader<T> {
    header: *const PoolHeader<T>,
    pub(crate) next_page: Option<Box<Page<T>>>,
    pub(crate) next_free_page: Cell<Option<*const Page<T>>>,
    first_free_slot: Cell<Index>,
    count: Cell<Index>,
}

union SlotUnion<T> {
    header: ManuallyDrop<PageHeader<T>>,
    slot: ManuallyDrop<Slot<T>>,
}

pub(crate) struct Page<T> {
    slots: Vec<SlotUnion<T>>,
}

impl<T> Page<T> {
    #[must_use]
    pub fn new(header: *const PoolHeader<T>, cap: Index, next_page: Option<Box<Page<T>>>) -> Self {
        let mut slots = Vec::with_capacity(cap as usize + 1);

        slots.push(SlotUnion {
            header: ManuallyDrop::new(PageHeader {
                header,
                next_page,
                next_free_page: Default::default(),
                first_free_slot: Default::default(),
                count: Default::default(),
            }),
        });

        for i in 1..slots.capacity() {
            slots.push(SlotUnion {
                slot: ManuallyDrop::new(Slot {
                    item: MaybeUninit::uninit().into(),
                    version: Cell::new(NonZeroUsize::new(1).unwrap()),
                    count: 0.into(),
                    index: (i as Index).into(),
                }),
            })
        }

        Self { slots }
    }

    #[must_use]
    pub(crate) fn header(&self) -> &PageHeader<T> {
        unsafe { self.slots.get_unchecked(0).header.deref() }
    }

    #[must_use]
    pub(crate) unsafe fn get(&self, index: Index) -> Option<&Slot<T>> {
        let slot = self.slots.get_unchecked(index as usize + 1);

        if slot.slot.is_free() {
            None
        } else {
            Some(&slot.slot)
        }
    }

    #[must_use]
    pub(crate) fn len(&self) -> Index {
        self.header().count.get()
    }

    #[must_use]
    pub(crate) fn capacity(&self) -> Index {
        self.slots.len() as Index - 1
    }

    #[must_use]
    pub(crate) unsafe fn insert(&self, value: T) -> &Slot<T> {
        let header = self.header();
        let index = header.first_free_slot.get() + 1;
        let slot = &self.slots.get_unchecked(index as usize).slot;
        slot.set_value(value);
        header.first_free_slot.set(slot.index.get());
        header.count.add(1);
        slot.index.set(index - 1);
        slot
    }
}
