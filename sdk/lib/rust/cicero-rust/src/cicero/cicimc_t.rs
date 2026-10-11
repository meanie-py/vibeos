//! `CIC_IMCC_LOCK<T>` / `CicIMCCLock<T>` (base/ctf/cicero/cicimc.h), hand-ported: the generics
//! `rustconv::cpp_templates` maps every instantiation onto, here and in msctfime.
//!
//! The only member that names T is `T_DATA *m_pIMCC`, a pointer, so the layout is the same
//! for every T. The lock is an RAII guard in C++ (the destructor unlocks), and so it is here:
//! a `CicIMCCLock` local unlocks when it goes out of scope, as the C++ object did.
use crate::prelude::*;

#[repr(C)]
pub struct CIC_IMCC_LOCK<T> {
    pub m_pIMCC: *mut T,
    pub m_hIMCC: HIMCC,
    pub m_hr: HRESULT,
}

#[repr(C)]
pub struct CicIMCCLock<T> {
    pub _base: CIC_IMCC_LOCK<T>,
}

impl<T> CicIMCCLock<T> {
    /// `CicIMCCLock(HIMCC)`: the base sets the handle, then the lock is taken.
    pub unsafe fn __ctor_0(&mut self, hIMCC: HIMCC) {
        self._base.m_pIMCC = core::ptr::null_mut();
        self._base.m_hr = S_OK;
        self._base.m_hIMCC = hIMCC;
        self._base.m_hr = Self::_LockIMCC(self._base.m_hIMCC, &mut self._base.m_pIMCC);
    }

    pub unsafe fn unlock(&mut self) {
        if !self._base.m_pIMCC.is_null() {
            Self::_UnlockIMCC(self._base.m_hIMCC);
            self._base.m_pIMCC = core::ptr::null_mut();
        }
    }

    /// `operator T_DATA*()`.
    pub fn op_to_ptr(&self) -> *mut T {
        self._base.m_pIMCC
    }
    /// `if (!lock)`: the conversion to `T_DATA*`, tested.
    pub fn operator_bool(&self) -> bool {
        !self._base.m_pIMCC.is_null()
    }
    /// The same test, as cppapp's lowering spells a pointer conversion used as a condition.
    pub fn __bool(&self) -> bool {
        self.operator_bool()
    }
    pub unsafe fn get(&self) -> &mut T {
        &mut *self._base.m_pIMCC
    }

    unsafe fn _LockIMCC(hIMCC: HIMCC, pptr: &mut *mut T) -> HRESULT {
        if hIMCC.is_null() {
            return E_INVALIDARG;
        }
        *pptr = ImmLockIMCC(hIMCC) as *mut T;
        if pptr.is_null() {
            E_FAIL
        } else {
            S_OK
        }
    }
    unsafe fn _UnlockIMCC(hIMCC: HIMCC) -> HRESULT {
        if ImmUnlockIMCC(hIMCC) == 0 {
            return if GetLastError() != 0 { E_FAIL } else { S_OK };
        }
        S_OK
    }
}

/// C++ reaches the base's members as the lock's own (`imcLock.m_hr`), as for any class with
/// a base; 58 msctfime sites spell it that way.
impl<T> core::ops::Deref for CicIMCCLock<T> {
    type Target = CIC_IMCC_LOCK<T>;
    fn deref(&self) -> &CIC_IMCC_LOCK<T> {
        &self._base
    }
}
impl<T> core::ops::DerefMut for CicIMCCLock<T> {
    fn deref_mut(&mut self) -> &mut CIC_IMCC_LOCK<T> {
        &mut self._base
    }
}

impl<T> Drop for CicIMCCLock<T> {
    fn drop(&mut self) {
        unsafe { self.unlock() }
    }
}

/// `CicIMCCLock<T> lock(hIMCC);` as a value.
pub unsafe fn CicIMCCLock__make_0<T>(hIMCC: HIMCC) -> CicIMCCLock<T> {
    let mut lock = CicIMCCLock {
        _base: CIC_IMCC_LOCK {
            m_pIMCC: core::ptr::null_mut(),
            m_hIMCC: core::ptr::null_mut(),
            m_hr: S_OK,
        },
    };
    lock.__ctor_0(hIMCC);
    lock
}

/// cppapp's two-step spelling of the same construction (`X__zeroed()` then `.__ctor_0(h)`), as
/// for every non-template class. All-zero is a valid unlocked state: a null `m_pIMCC` makes the
/// destructor a no-op, exactly as for a C++ lock whose `_LockIMCC` failed.
pub unsafe fn CicIMCCLock__zeroed<T>() -> CicIMCCLock<T> {
    core::mem::zeroed()
}
