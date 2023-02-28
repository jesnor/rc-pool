use std::{
    cell::Cell,
    ops::{AddAssign, DivAssign, MulAssign, Neg, RemAssign, SubAssign},
};

pub trait CellTrait<T> {
    fn as_ptr(&self) -> *mut T;
    fn set(&self, value: T);

    fn take(&self) -> T
    where
        T: Default;

    fn get_clone(&self) -> T
    where
        T: Clone + Default,
    {
        let v = self.take();
        self.set(v.clone());
        v
    }

    fn add<Rhs>(&self, rhs: Rhs)
    where
        T: AddAssign<Rhs> + Default,
    {
        let mut v = self.take();
        v.add_assign(rhs);
        self.set(v)
    }

    fn sub<Rhs>(&self, rhs: Rhs)
    where
        T: SubAssign<Rhs> + Default,
    {
        let mut v = self.take();
        v.sub_assign(rhs);
        self.set(v)
    }

    fn mul<Rhs>(&self, rhs: Rhs)
    where
        T: MulAssign<Rhs> + Default,
    {
        let mut v = self.take();
        v.mul_assign(rhs);
        self.set(v)
    }

    fn div<Rhs>(&self, rhs: Rhs)
    where
        T: DivAssign<Rhs> + Default,
    {
        let mut v = self.take();
        v.div_assign(rhs);
        self.set(v)
    }

    fn rem<Rhs>(&self, rhs: Rhs)
    where
        T: RemAssign<Rhs> + Default,
    {
        let mut v = self.take();
        v.rem_assign(rhs);
        self.set(v)
    }

    fn neg(&self)
    where
        T: Neg<Output = T> + Default,
    {
        self.set(self.take().neg())
    }
}

impl<T> CellTrait<T> for Cell<T> {
    fn as_ptr(&self) -> *mut T {
        self.as_ptr()
    }

    fn set(&self, value: T) {
        self.set(value)
    }

    fn take(&self) -> T
    where
        T: Default,
    {
        self.take()
    }
}
