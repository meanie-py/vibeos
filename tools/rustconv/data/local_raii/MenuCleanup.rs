// --local-raii MenuCleanup: hand-verified lowering (spliced verbatim; workers received this API as a sealed contract)
//
// Hand-verified Rust lowering of the file-local C++ RAII guard `MenuCleanup`
// (dll/win32/shell32/CDefView.cpp:131). Consumed by
//   rustconv translate --cpp-com --class CDefView \
//     --local-raii MenuCleanup=tools/rustconv/data/local_raii/MenuCleanup.rs
//
// C++ semantics preserved exactly:
//   ctor  — binds REFERENCES to the caller's two variables; copies nothing
//   dtor  — if (m_pCM)   { IUnknown_SetSite(m_pCM, NULL); m_pCM.Release(); }
//           if (m_hMenu) { DestroyMenu(m_hMenu); m_hMenu = NULL; }
//
// Both members are C++ REFERENCES (`CComPtr<IContextMenu>&`, `HMENU&`) and that is the entire
// point of the guard: it clears the CALLER's variables, not copies of them. All four call sites
// depend on it, and two bind CDefView's own fields — `MenuCleanup _(m_pCM, m_hContextMenu)` at
// :2308 and `MenuCleanup _(m_pFileMenu, Dummy)` at :2646.
//
// So this holds RAW POINTERS rather than `&mut`. A Rust `&mut` into a field of `*mut CDefView`
// would claim exclusive access to storage the surrounding worker keeps reaching through `this`
// for the whole guarded scope, which is exactly the aliasing the C++ does freely. The null
// checks on the pointers themselves cannot fire from correct code — a C++ reference is never
// null — and exist only because lowering a reference to a raw pointer removes the guarantee.
//
// Both slots end NULL and that is load-bearing, not tidiness: `CComPtr::Release()` nulls `p`
// the way ATL's does (atl.rs:314) and the C++ assigns `m_hMenu = NULL` explicitly, so a caller
// that re-tests either variable after the guarded scope sees the cleared value.
struct MenuCleanup {
    m_pCM: *mut CComPtr<IContextMenu>,
    m_hMenu: *mut HMENU,
}

impl MenuCleanup {
    unsafe fn new(pCM: *mut CComPtr<IContextMenu>, menu: *mut HMENU) -> MenuCleanup {
        MenuCleanup {
            m_pCM: pCM,
            m_hMenu: menu,
        }
    }
}

impl Drop for MenuCleanup {
    fn drop(&mut self) {
        unsafe {
            if !self.m_pCM.is_null() && !(*self.m_pCM).is_null() {
                IUnknown_SetSite((*self.m_pCM).as_ptr().cast(), core::ptr::null_mut());
                (*self.m_pCM).Release();
            }
            if !self.m_hMenu.is_null() && !(*self.m_hMenu).is_null() {
                DestroyMenu(*self.m_hMenu);
                *self.m_hMenu = core::ptr::null_mut();
            }
        }
    }
}
