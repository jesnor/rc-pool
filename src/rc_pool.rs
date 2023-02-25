use crate::cell_trait::CellTrait;
use crate::rc_traits::{StrongRefTrait, WeakRefTrait};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::DerefMut;
use std::{
    cell::{Cell, UnsafeCell},
    mem::MaybeUninit,
    ops::Deref,
};

const MUT_REF_COUNT: u32 = u32::MAX;

type Index = u32;
type Version = u32;
type Count = u32;

pub struct Slot<T> {
    value: UnsafeCell<MaybeUninit<T>>,
    version: Cell<Version>,
    count: Cell<Count>,
    index: Cell<Index>,
}

impl<T> Slot<T> {
    #[must_use]
    unsafe fn get(&self) -> &T {
        debug_assert!(self.count.get() > 0);
        (*self.value.get()).assume_init_ref()
    }

    #[must_use]
    #[allow(clippy::mut_from_ref)]
    unsafe fn get_mut(&self) -> &mut T {
        debug_assert!(self.count.get() == 1);
        (*self.value.get()).assume_init_mut()
    }

    unsafe fn set_value(&self, value: T) {
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

    fn is_free(&self) -> bool {
        self.version.get() & 1 == 0
    }

    fn incr_version(&self) {
        self.version.set(self.version.get().wrapping_add(1));
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
    fn new(slot: &'t Slot<T>) -> Self {
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

impl<'t, T> Debug for StrongRef<'t, T> {
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

pub struct WeakRef<'t, T> {
    slot: &'t Slot<T>,
    version: Version,
}

impl<'t, T> WeakRef<'t, T> {
    #[must_use]
    fn new(slot: &'t Slot<T>) -> Self {
        Self {
            slot,
            version: slot.version.get(),
        }
    }
}

impl<'t, T> WeakRefTrait for WeakRef<'t, T> {
    type Target = T;
    type Strong = StrongRef<'t, T>;

    #[must_use]
    fn strong(&self) -> Option<StrongRef<'t, T>> {
        if self.is_valid() {
            assert_ne!(self.slot.count.get(), MUT_REF_COUNT);
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

impl<'t, T> Debug for WeakRef<'t, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakRef")
            .field("slot", &(self.slot as *const Slot<T>))
            .field("version", &self.version)
            .finish()
    }
}

pub struct RcPool<T, A> {
    slots: A,
    _phantom: PhantomData<fn(T) -> T>,
}

pub type VecRcPool<T> = RcPool<T, Vec<Slot<T>>>;
pub type ArrayRcPool<T, const CAP: usize> = RcPool<T, [Slot<T>; CAP]>;

impl<T> RcPool<T, Vec<Slot<T>>> {
    #[must_use]
    pub fn new_vec(cap: usize) -> Self {
        let mut v = Vec::with_capacity(cap);

        for i in 0..v.capacity() {
            v.push(Slot {
                value: MaybeUninit::uninit().into(),
                version: 0.into(),
                count: 0.into(),
                index: (i as Index + 1).into(),
            })
        }

        Self::new(v)
    }
}

impl<T, A: AsRef<[Slot<T>]>> RcPool<T, A> {
    #[must_use]
    pub fn new(slots: A) -> Self {
        assert!(!slots.as_ref().is_empty());

        for (i, slot) in slots.as_ref().iter().enumerate() {
            slot.index.set(i as u32 + 1);
        }

        Self {
            slots,
            _phantom: Default::default(),
        }
    }

    fn head(&self) -> &Slot<T> {
        unsafe { self.slots.as_ref().get_unchecked(0) }
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<StrongRef<T>> {
        self.slots.as_ref().get(index).and_then(|slot| {
            if slot.is_free() {
                None
            } else {
                Some(StrongRef::new(slot))
            }
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = StrongRef<T>> {
        self.slots.as_ref().iter().filter_map(|slot| {
            if slot.is_free() {
                None
            } else {
                Some(StrongRef::new(slot))
            }
        })
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.head().count.get() as usize
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn capacity(&self) -> usize {
        self.slots.as_ref().len()
    }

    #[must_use]
    pub fn insert(&self, value: T) -> Option<StrongRef<T>> {
        let head = self.head();
        let index = head.index.get();

        if let Some(slot) = self.slots.as_ref().get(index as usize) {
            unsafe { slot.set_value(value) };
            head.index.set(slot.index.get());
            head.count.add(1);
            Some(StrongRef::new(slot))
        } else {
            None
        }
    }

    pub fn remove(&self, r: &WeakRef<T>) {
        if r.is_valid() && r.slot.count.get() == 0 {
            unsafe { r.slot.drop_value() };
            let index = r.slot.index.get();
            let head = self.head();
            r.slot.index.set(head.index.get());
            head.index.set(index);
            head.count.sub(1);
        }
    }
}

impl<T, A: AsRef<[Slot<T>]> + Default> Default for RcPool<T, A> {
    fn default() -> Self {
        Self::new(A::default())
    }
}

pub type RcVecPool<T> = RcPool<T, Vec<Slot<T>>>;
