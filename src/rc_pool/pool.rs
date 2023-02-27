use super::page::Page;
use crate::{Index, StrongRef};
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
    page_size: Index,
}

impl<T, const MANUAL_DROP: bool> RcPool<T, MANUAL_DROP> {
    #[must_use]
    pub fn new(page_size: Index) -> Self {
        let header: Box<PoolHeader<T>> = Box::new(PoolHeader {
            first_page: UnsafeCell::new(None),
            first_free_page: Cell::new(null()),
        });

        let first_page = Box::new(Page::new(header.deref() as *const _, page_size, None));

        unsafe {
            *header.first_page.get() = Some(first_page);

            header
                .first_free_page
                .set(header.first_page().deref() as *const _);
        }

        Self { header, page_size }
    }

    fn add_page(&self) -> &Page<T> {
        unsafe {
            let first_page = (*self.header.first_page.get()).take();

            let new_page = Box::new(Page::new(
                self.header.deref() as *const _,
                self.page_size,
                first_page,
            ));

            self.header
                .first_free_page
                .set(new_page.deref() as *const _);

            *self.header.first_page.get() = Some(new_page);
            self.header.first_page()
        }
    }

    #[must_use]
    pub fn insert(&self, value: T) -> StrongRef<T, MANUAL_DROP> {
        let mut page = unsafe { self.header.first_free_page() };

        loop {
            if page.len() < page.capacity() {
                let r = unsafe { page.insert(value) };
                self.header.first_free_page.set(page as *const _); // Set this page as the first with free slots
                return StrongRef::new(r);
            }

            if let Some(next_free_page) = page.header().next_free_page.get() {
                page = unsafe { &*next_free_page };
            } else {
                break;
            }
        }

        // No free slot in any page, add a new page
        StrongRef::new(unsafe { self.add_page().insert(value) })
    }

    fn first_page(&self) -> &Page<T> {
        unsafe { &*self.header.first_page.get() }
            .as_deref()
            .unwrap()
    }

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
