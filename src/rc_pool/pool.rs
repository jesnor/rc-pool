use super::page::Page;
use crate::{Either, Index, StrongRef};
use std::marker::PhantomData;
use std::ptr::null;
use std::{
    cell::{Cell, UnsafeCell},
    ops::Deref,
};

pub(crate) struct PoolHeader<T> {
    first_page: UnsafeCell<Option<Box<Page<T>>>>,
    first_free_page: Cell<*const Page<T>>,
}

impl<T> PoolHeader<T> {
    unsafe fn first_page(&self) -> &Page<T> {
        (*self.first_page.get()).as_ref().unwrap()
    }

    unsafe fn first_free_page(&self) -> &Page<T> {
        &*self.first_free_page.get()
    }
}

pub struct RcPool<T, const MANUAL_DROP: bool> {
    header: Box<PoolHeader<T>>,
    page_len: Cell<Index>,
}

impl<T, const MANUAL_DROP: bool> RcPool<T, MANUAL_DROP> {
    #[must_use]
    pub fn new(page_len: Index) -> Self {
        let header: Box<PoolHeader<T>> = Box::new(PoolHeader {
            first_page: UnsafeCell::new(None),
            first_free_page: Cell::new(null()),
        });

        let first_page = Box::new(Page::new(header.deref() as *const _, page_len, None));

        unsafe {
            *header.first_page.get() = Some(first_page);

            header
                .first_free_page
                .set(header.first_page().deref() as *const _);
        }

        Self {
            header,
            page_len: page_len.into(),
        }
    }

    /// Sets the number of slots of newly created pages
    pub fn set_page_len(&self, page_size: Index) {
        self.page_len.set(page_size)
    }

    /// Returns the number of slots for newly created pages
    #[must_use]
    pub fn page_len(&self) -> Index {
        self.page_len.get()
    }

    fn add_page(&self, page_size: Index) -> &Page<T> {
        unsafe {
            let first_page = (*self.header.first_page.get()).take();

            let new_page = Box::new(Page::new(
                self.header.deref() as *const _,
                page_size,
                first_page,
            ));

            self.header
                .first_free_page
                .set(new_page.deref() as *const _);

            *self.header.first_page.get() = Some(new_page);
            self.header.first_page()
        }
    }

    /// Inserts a new item into the pool
    /// If there is a free slot, creates and returns a strong reference to that slot,
    /// otherwise returns the item
    #[must_use]
    pub fn try_insert(&self, value: T) -> Either<StrongRef<T, MANUAL_DROP>, T> {
        let mut page = unsafe { self.header.first_free_page() };

        loop {
            if page.len() < page.capacity() {
                let r = unsafe { page.insert(value) };
                self.header.first_free_page.set(page as *const _); // Set this page as the first with free slots
                return Either::Left(StrongRef::new(r));
            }

            if let Some(next_free_page) = page.header().next_free_page.get() {
                page = unsafe { &*next_free_page };
            } else {
                break;
            }
        }

        Either::Right(value)
    }

    /// Inserts a new item into the pool
    /// If there is a free slot, creates and returns a strong reference to that slot,
    /// otherwise a new slot page of size [page_len()] will be added and the item is placed inside it
    #[must_use]
    pub fn insert(&self, value: T) -> StrongRef<T, MANUAL_DROP> {
        match self.try_insert(value) {
            Either::Left(r) => r,

            Either::Right(v) => {
                StrongRef::new(unsafe { self.add_page(self.page_len.get()).insert(v) })
            }
        }
    }

    fn first_page(&self) -> &Page<T> {
        unsafe { &*self.header.first_page.get() }
            .as_deref()
            .unwrap()
    }

    #[must_use]
    pub fn iter(&self) -> RcPoolIterator<T, MANUAL_DROP> {
        RcPoolIterator {
            page: Some(self.first_page() as *const _),
            index: 0,
            phantom: PhantomData::default(),
        }
    }
}

pub struct RcPoolIterator<'t, T, const MANUAL_DROP: bool> {
    page: Option<*const Page<T>>,
    index: Index,
    phantom: PhantomData<&'t mut ()>,
}

impl<'t, T: 't, const MANUAL_DROP: bool> Iterator for RcPoolIterator<'t, T, MANUAL_DROP> {
    type Item = StrongRef<'t, T, MANUAL_DROP>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(p) = self.page {
            let page = unsafe { &*p };

            if self.index >= page.capacity() {
                self.page = page
                    .header()
                    .next_page
                    .as_ref()
                    .map(|p| p.deref() as *const _);

                self.index = 0;
            } else if let Some(r) = unsafe { page.get(self.index) } {
                self.index += 1;
                return Some(StrongRef::new(r));
            } else {
                self.index += 1;
            }
        }

        None
    }
}
