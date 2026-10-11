//! `CicArray<T>` (base/ctf/cicero/cicarray.h), hand-ported: the generic that
//! `rustconv::cpp_templates` maps every `CicArray<...>` base, field and global onto, here
//! and in msctf/msutb/msctfime.
//!
//! `class CicArray : protected CicArrayBase` adds no fields, so every instantiation is the
//! base's layout: vptr, `m_pb`, `m_cItems`, `m_cbItem`, `m_cCapacity`. The element type only
//! changes how `m_pb` is read, which is what `PhantomData` records.
use crate::prelude::*;
use core::marker::PhantomData;

#[repr(C)]
pub struct CicArray<T> {
    pub __base0: CicArrayBase,
    _item: PhantomData<T>,
}

impl<T> CicArray<T> {
    /// `CicArray() : CicArrayBase(sizeof(T_ITEM))`.
    pub unsafe fn __ctor_0(&mut self) {
        self.__base0.__ctor_0(core::mem::size_of::<T>());
    }

    pub fn data(&self) -> *mut T {
        self.__base0.m_pb as *mut T
    }
    pub fn size(&self) -> size_t {
        self.__base0.m_cItems
    }
    pub fn empty(&self) -> bool {
        self.size() == 0
    }
    pub unsafe fn clear(&mut self) {
        cicMemFree(self.__base0.m_pb as LPVOID);
        self.__base0.m_pb = core::ptr::null_mut();
        self.__base0.m_cItems = 0;
        self.__base0.m_cCapacity = 0;
    }

    /// `operator[]`: by `m_cbItem`, as the C++ does, not by `size_of::<T>()` (equal unless
    /// the array was built by C++ code with a different item size).
    pub unsafe fn at(&self, iItem: size_t) -> *mut T {
        self.__base0.m_pb.add(iItem * self.__base0.m_cbItem) as *mut T
    }

    pub unsafe fn Append(&mut self, cGrow: size_t) -> *mut T {
        self.__base0.Append(cGrow) as *mut T
    }
    pub unsafe fn Insert(&mut self, iItem: size_t, cGrow: size_t) -> BOOL {
        self.__base0.Insert(iItem, cGrow)
    }
    pub unsafe fn Remove(&mut self, iItem: size_t, cRemove: size_t) {
        self.__base0.Remove(iItem, cRemove)
    }

    /// `Add(const T_ITEM&)`: by value here, the reference only spared C++ a copy.
    pub unsafe fn Add(&mut self, item: T) -> BOOL {
        let pItem = self.Append(1);
        if pItem.is_null() {
            return FALSE as BOOL;
        }
        core::ptr::write(pItem, item);
        TRUE as BOOL
    }

    /// The element pointer, as the translated `m_array[i]` spells it (`.add(i)`).
    pub unsafe fn add(&self, iItem: size_t) -> *mut T {
        self.at(iItem)
    }
}

impl<T: PartialEq> CicArray<T> {
    /// `Find(const T_ITEM&)`, by value for the same reason as `Add`.
    pub unsafe fn Find(&self, item: T) -> isize {
        for iItem in 0..self.__base0.m_cItems {
            if *self.at(iItem) == item {
                return iItem as isize;
            }
        }
        -1
    }
}

impl<T> core::ops::Index<size_t> for CicArray<T> {
    type Output = T;
    fn index(&self, iItem: size_t) -> &T {
        unsafe { &*self.at(iItem) }
    }
}
impl<T> core::ops::IndexMut<size_t> for CicArray<T> {
    fn index_mut(&mut self, iItem: size_t) -> &mut T {
        unsafe { &mut *self.at(iItem) }
    }
}

// `class CicArray : protected CicArrayBase`: its own members (and CUIFObjectArray's, which
// derives from it) reach `m_cItems`/`m_pb` as their own.
impl<T> core::ops::Deref for CicArray<T> {
    type Target = CicArrayBase;
    fn deref(&self) -> &CicArrayBase {
        &self.__base0
    }
}
impl<T> core::ops::DerefMut for CicArray<T> {
    fn deref_mut(&mut self) -> &mut CicArrayBase {
        &mut self.__base0
    }
}
