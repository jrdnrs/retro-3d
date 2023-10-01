use core::{
    mem::{transmute, MaybeUninit},
    ops::{Index, IndexMut},
};

pub struct FixedVec<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> FixedVec<T, N> {
    pub fn new() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.len == N {
            panic!("StackVec is full");
        }

        self.data[self.len] = MaybeUninit::new(value);
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;

        return Some(unsafe { self.data[self.len].assume_init_read() });
    }

    pub fn swap_remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }

        self.len -= 1;

        self.data.swap(index, self.len);

        return Some(unsafe { self.data[self.len].assume_init_read() });
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> core::slice::Iter<T> {
        self.as_slice().iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<T> {
        self.as_mut_slice().iter_mut()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }

        return Some(unsafe { self.data[index].assume_init_ref() });
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        self.data[index].assume_init_ref()
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }

        return Some(unsafe { self.data[index].assume_init_mut() });
    }

    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        self.data[index].assume_init_mut()
    }

    pub fn last(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }

        return Some(unsafe { self.data[self.len - 1].assume_init_ref() });
    }

    pub unsafe fn last_unchecked(&self) -> &T {
        self.data[self.len - 1].assume_init_ref()
    }

    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.len == 0 {
            return None;
        }

        return Some(unsafe { self.data[self.len - 1].assume_init_mut() });
    }

    pub unsafe fn last_unchecked_mut(&mut self) -> &mut T {
        self.data[self.len - 1].assume_init_mut()
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { transmute(&self.data[0..self.len]) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { transmute(&mut self.data[0..self.len]) }
    }
}

impl<T, const N: usize> Index<usize> for FixedVec<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T, const N: usize> IndexMut<usize> for FixedVec<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}
