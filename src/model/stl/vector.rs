use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::slice;

use libc::size_t;

use super::bindings::root::rust::*;
use super::memory::UniquePtr;

#[repr(C)]
pub struct Vector<T>(dummy_vector, PhantomData<T>);

pub trait VectorSlice {
    type Item;

    fn get_ptr(&self) -> *const Self::Item {
        self.as_slice().as_ptr()
    }

    fn get_mut_ptr(&mut self) -> *mut Self::Item {
        self.as_mut_slice().as_mut_ptr()
    }

    fn size(&self) -> usize {
        self.as_slice().len()
    }

    fn as_slice(&self) -> &[Self::Item] {
        unsafe { slice::from_raw_parts(self.get_ptr(), self.size()) }
    }

    fn as_mut_slice(&mut self) -> &mut [Self::Item] {
        unsafe { slice::from_raw_parts_mut(self.get_mut_ptr(), self.size()) }
    }
}

macro_rules! add_impl {
    ($($t:ty)*) => ($(
        impl fmt::Debug for $t
        where
            <$t as VectorSlice>::Item: fmt::Debug,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.as_slice().iter()).finish()
            }
        }

        impl Deref for $t {
            type Target = [<$t as VectorSlice>::Item];

            fn deref(&self) -> &Self::Target {
                self.as_slice()
            }
        }

        impl DerefMut for $t {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.as_mut_slice()
            }
        }

        impl Index<usize> for $t {
            type Output = <$t as VectorSlice>::Item;

            fn index(&self, index: usize) -> &Self::Output {
                &self.as_slice()[index]
            }
        }

        impl IndexMut<usize> for $t {
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                &mut self.as_mut_slice()[index]
            }
        }

        impl<'a> IntoIterator for &'a $t {
            type Item = &'a <$t as VectorSlice>::Item;
            type IntoIter = slice::Iter<'a, <$t as VectorSlice>::Item>;

            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }

        impl<'a> IntoIterator for &'a mut $t {
            type Item = &'a mut <$t as VectorSlice>::Item;
            type IntoIter = slice::IterMut<'a, <$t as VectorSlice>::Item>;

            fn into_iter(self) -> Self::IntoIter {
                self.iter_mut()
            }
        }
    )*)
}

pub trait VectorRemove: VectorSlice {
    fn erase_range(&mut self, offset: usize, len: usize) {
        for i in (offset..offset + len).rev() {
            self.erase(i);
        }
    }

    fn pop_back(&mut self) {
        assert!(self.size() > 0);
        self.erase(self.size() - 1);
    }

    fn erase(&mut self, index: usize) {
        self.erase_range(index, 1);
    }

    fn clear(&mut self) {
        self.erase_range(0, self.size());
    }

    fn retain(&mut self, pred: fn(usize, &Self::Item) -> bool) {
        let removed: Vec<_> = self
            .as_slice()
            .iter()
            .enumerate()
            .filter_map(|(i, op)| if pred(i, op) { None } else { Some(i) })
            .collect();

        for i in removed.into_iter().rev() {
            self.erase(i);
        }
    }

    fn truncate(&mut self, size: usize) {
        assert!(size <= self.size());
        self.erase_range(size, self.size() - size);
    }
}

pub trait VectorInsert<T>: VectorRemove {
    fn push_back(&mut self, v: T);

    fn assign<I: IntoIterator<Item = T>>(&mut self, vs: I) {
        self.clear();
        for v in vs {
            self.push_back(v);
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct VectorOfBool(vector_of_bool);

#[repr(C)]
#[derive(Debug)]
pub struct VectorOfUniquePtr<T>(dummy_vector, PhantomData<T>);

#[derive(Debug)]
pub struct InnerIndex(pub usize);

impl<T> Index<InnerIndex> for Vector<UniquePtr<T>> {
    type Output = T;

    fn index(&self, index: InnerIndex) -> &Self::Output {
        let index = index.0 as size_t;
        unsafe {
            let ptr = cpp!([self as "const std::vector<std::unique_ptr<void>>*", index as "size_t"]
                            -> *const crate::model::OperatorCodeT as "const void*" {
                return (*self)[index].get();
            }) as *const Self::Output;

            ptr.as_ref().unwrap()
        }
    }
}

impl<T> IndexMut<InnerIndex> for Vector<UniquePtr<T>> {
    fn index_mut(&mut self, index: InnerIndex) -> &mut Self::Output {
        let index = index.0 as size_t;
        unsafe {
            let ptr = cpp!([self as "std::vector<std::unique_ptr<void>>*", index as "size_t"]
                            -> *mut crate::model::OperatorCodeT as "void*" {
                return (*self)[index].get();
            }) as *mut Self::Output;

            ptr.as_mut().unwrap()
        }
    }
}