// rustconv-assembly: whole-tu
use crate::prelude::*;
// The macro-generated half of cpp.c: the vtables, the RTTI/throw descriptors, the
// `rtti_rva*` helpers and the seven `CREATE_*` bodies. See cpp_rtti.rs for why they
// cannot come from the translator.
use crate::cpp_rtti::*;
use msvcrt_sys::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type bad_cast = exception;
pub type bad_typeid = exception;
pub type __non_rtti_object = exception;

// `type_info140` and `type_info_entry` were here, and are gone on purpose: their only users
// are the `_MSVCR_VER >= 140 || defined(__UCRTSUPPORT__)` bodies, which msvcrt does not
// compile and which this wave deliberately did not translate. Keeping them would have left
// the only reference to `SLIST_ENTRY` in the file — a type the island does not carry,
// because nothing msvcrt builds needs it.
extern "C" {
    /// `abort` is exit.c's export — `terminate` falls through to it. Reached by SYMBOL
    /// rather than as a `use super::exit::abort` Rust path, which is both more faithful and
    /// what makes the module linkable in a host harness. Faithful: the C calls `abort()`
    /// across a TU boundary and the linker resolves it, so in msvcrt.dll this binds to
    /// exactly the same `#[no_mangle]` body the `use` did. Linkable: exit.c defines `exit`,
    /// `_exit` and `abort` under their plain names, so a harness that had to switch on the
    /// `conio-exit` feature just to satisfy one call would hand the Rust test runtime's own
    /// exit path to a body that calls `ExitProcess` — see that feature's note in Cargo.toml.
    /// The path form made `cpp` un-buildable without `conio-exit` and hid that behind an
    /// unresolved-import error rather than a link error.
    fn abort();
}

unsafe extern "C" fn get_vtable(obj: *mut core::ffi::c_void) -> *const vtable_ptr {
    *(obj as *const *const vtable_ptr)
}
unsafe extern "C" fn get_obj_locator(cppobj: *mut core::ffi::c_void) -> *const rtti_object_locator {
    let vtable = get_vtable(cppobj);
    // C: `(const rtti_object_locator *)vtable[-1]`. `vtable_ptr` is bindgen's
    // `Option<unsafe extern "C" fn()>`, and Rust has no cast from that to a data pointer —
    // so read the slot AS a pointer rather than reinterpreting the loaded value.
    *(vtable.offset(-1) as *const *const rtti_object_locator)
}
unsafe extern "C" fn get_obj_locator_base(ptr: *const rtti_object_locator) -> usize {
    // C: `#ifdef RTTI_USE_RVA`, and cxx.h defines that macro for everything EXCEPT
    // `__i386__` — so the arm is live on amd64/arm64 and dead on i386.
    //
    // The model wrote `all(target_pointer_width = "32", feature = "rtti_rva")`: inverted
    // (32-bit is precisely where the arm is dead) AND gated on a cargo feature that does
    // not exist, so the condition was false on all three arches and every one of them
    // silently took the i386 path. `object_locator` is the sixth field that only the RVA
    // struct has, so this branch is the ONLY thing that recovers the image base for a
    // locator the loader relocated; without it every RTTI lookup on 64-bit resolves
    // against base 0.
    #[cfg(not(target_arch = "x86"))]
    {
        if (*ptr).signature != 0 {
            return (ptr as usize).wrapping_sub((*ptr).object_locator as usize);
        }
    }
    rtti_rva_base(ptr)
}
unsafe extern "C" fn dump_obj_locator(ptr: *const rtti_object_locator) {
    let mut i: i32;
    let base: usize = get_obj_locator_base(ptr);
    let h: *const rtti_object_hierarchy = rtti_rva((*ptr).type_hierarchy, base);
    let type_descriptor: *const type_info = rtti_rva((*ptr).type_descriptor, base);

    i = 0;
    while i < (*h).array_len {
        let base_array: *const rtti_base_array = rtti_rva((*h).base_classes, base);
        let bases: *const rtti_base_descriptor = rtti_rva((*base_array).bases[i as usize], base);

        i += 1;
    }
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn exception_ctor(
    _this: *mut exception,
    name: *const *const i8,
) -> *mut exception {
    return __exception_ctor(_this, *name, core::ptr::addr_of!(exception_vtable));
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn exception_ctor(
    _this: *mut exception,
    name: *const *const i8,
) -> *mut exception {
    return __exception_ctor(_this, *name, core::ptr::addr_of!(exception_vtable));
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn exception_ctor_noalloc(
    _this: *mut exception,
    name: *mut *mut i8,
    noalloc: i32,
) -> *mut exception {
    (*_this).vtable = core::ptr::addr_of!(exception_vtable) as *const vtable_ptr;
    (*_this).name = *name;
    (*_this).do_free = 0; // C: FALSE (glob-ambiguous here)
    _this
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn exception_ctor_noalloc(
    _this: *mut exception,
    name: *mut *mut i8,
    noalloc: i32,
) -> *mut exception {
    (*_this).vtable = core::ptr::addr_of!(exception_vtable) as *const vtable_ptr;
    (*_this).name = *name;
    (*_this).do_free = 0; // C: FALSE (glob-ambiguous here)
    _this
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn exception_default_ctor(_this: *mut exception) -> *mut exception {
    return __exception_ctor(_this, core::ptr::null_mut(), core::ptr::addr_of!(exception_vtable) as *const _);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn exception_default_ctor(_this: *mut exception) -> *mut exception {
    return __exception_ctor(_this, core::ptr::null_mut(), core::ptr::addr_of!(exception_vtable) as *const _);
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn exception_opequals(
    _this: *mut exception,
    rhs: *const exception,
) -> *mut exception {
    if _this != rhs as *mut exception {
        exception_dtor(_this);
        exception_copy_ctor(_this, rhs);
    }
    _this
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn exception_opequals(
    _this: *mut exception,
    rhs: *const exception,
) -> *mut exception {
    if _this != rhs as *mut exception {
        exception_dtor(_this);
        exception_copy_ctor(_this, rhs);
    }
    _this
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn exception_scalar_dtor(_this: *mut exception, flags: u32) -> *mut core::ffi::c_void {
    exception_dtor(_this);
    if flags & 1 != 0 {
        operator_delete((_this).cast());
    }
    _this as *mut core::ffi::c_void
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn exception_scalar_dtor(_this: *mut exception, flags: u32) -> *mut core::ffi::c_void {
    exception_dtor(_this);
    if flags & 1 != 0 {
        operator_delete((_this).cast());
    }
    _this as *mut core::ffi::c_void
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn bad_typeid_copy_ctor(_this: *mut bad_typeid, rhs: *const bad_typeid) -> *mut bad_typeid {
    return __exception_copy_ctor(_this, rhs, core::ptr::addr_of!(bad_typeid_vtable));
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn bad_typeid_copy_ctor(_this: *mut bad_typeid, rhs: *const bad_typeid) -> *mut bad_typeid {
    return __exception_copy_ctor(_this, rhs, core::ptr::addr_of!(bad_typeid_vtable));
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn bad_typeid_ctor(
    _this: *mut bad_typeid,
    name: *const i8,
) -> *mut bad_typeid {
    return __exception_ctor(_this, name, core::ptr::addr_of!(bad_typeid_vtable));
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn bad_typeid_ctor(
    _this: *mut bad_typeid,
    name: *const i8,
) -> *mut bad_typeid {
    return __exception_ctor(_this, name, core::ptr::addr_of!(bad_typeid_vtable));
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn bad_typeid_default_ctor(
    _this: *mut bad_typeid,
) -> *mut bad_typeid {
    return bad_typeid_ctor(_this, b"bad typeid\0".as_ptr() as *const i8);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn bad_typeid_default_ctor(
    _this: *mut bad_typeid,
) -> *mut bad_typeid {
    return bad_typeid_ctor(_this, b"bad typeid\0".as_ptr() as *const i8);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn bad_typeid_dtor(_this: *mut bad_typeid) {
    // TRACE("(%p)\n", _this);
    exception_dtor(_this);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn bad_typeid_dtor(_this: *mut bad_typeid) {
    // TRACE("(%p)\n", _this);
    exception_dtor(_this);
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn bad_typeid_opequals(
    _this: *mut bad_typeid,
    rhs: *const bad_typeid,
) -> *mut bad_typeid {
    exception_opequals(_this as *mut exception, rhs as *const exception);
    return _this;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn bad_typeid_opequals(
    _this: *mut bad_typeid,
    rhs: *const bad_typeid,
) -> *mut bad_typeid {
    exception_opequals(_this as *mut exception, rhs as *const exception);
    return _this;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn bad_typeid_vector_dtor(_this: *mut bad_typeid, flags: u32) -> *mut core::ffi::c_void {
    if flags & 2 != 0 {
        let ptr = (_this as *mut INT_PTR).offset(-(1 as isize));
        let mut i = *ptr - 1;
        while i >= 0 {
            bad_typeid_dtor(_this.offset(i as isize));
            i -= 1;
        }
        operator_delete(ptr as *mut core::ffi::c_void);
    } else {
        bad_typeid_dtor(_this);
        if flags & 1 != 0 {
            operator_delete(_this as *mut core::ffi::c_void);
        }
    }
    _this as *mut core::ffi::c_void
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn bad_typeid_vector_dtor(_this: *mut bad_typeid, flags: u32) -> *mut core::ffi::c_void {
    if flags & 2 != 0 {
        let ptr = (_this as *mut INT_PTR).offset(-(1 as isize));
        let mut i = *ptr - 1;
        while i >= 0 {
            bad_typeid_dtor(_this.offset(i as isize));
            i -= 1;
        }
        operator_delete(ptr as *mut core::ffi::c_void);
    } else {
        bad_typeid_dtor(_this);
        if flags & 1 != 0 {
            operator_delete(_this as *mut core::ffi::c_void);
        }
    }
    _this as *mut core::ffi::c_void
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn bad_typeid_scalar_dtor(
    _this: *mut bad_typeid,
    flags: u32,
) -> *mut core::ffi::c_void {
    bad_typeid_dtor(_this);
    if flags & 1 != 0 {
        operator_delete((_this).cast());
    }
    _this as *mut core::ffi::c_void
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn bad_typeid_scalar_dtor(
    _this: *mut bad_typeid,
    flags: u32,
) -> *mut core::ffi::c_void {
    bad_typeid_dtor(_this);
    if flags & 1 != 0 {
        operator_delete((_this).cast());
    }
    _this as *mut core::ffi::c_void
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn __non_rtti_object_copy_ctor(
    _this: *mut __non_rtti_object,
    rhs: *const __non_rtti_object,
) -> *mut __non_rtti_object {
    return __exception_copy_ctor(
        _this,
        rhs,
        core::ptr::addr_of!(__non_rtti_object_vtable) as *const vtable_ptr,
    );
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn __non_rtti_object_copy_ctor(
    _this: *mut __non_rtti_object,
    rhs: *const __non_rtti_object,
) -> *mut __non_rtti_object {
    return __exception_copy_ctor(
        _this,
        rhs,
        core::ptr::addr_of!(__non_rtti_object_vtable) as *const vtable_ptr,
    );
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn __non_rtti_object_ctor(
    _this: *mut __non_rtti_object,
    name: *const i8,
) -> *mut __non_rtti_object {
    return __exception_ctor(_this, name, core::ptr::addr_of!(__non_rtti_object_vtable));
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn __non_rtti_object_ctor(
    _this: *mut __non_rtti_object,
    name: *const i8,
) -> *mut __non_rtti_object {
    return __exception_ctor(_this, name, core::ptr::addr_of!(__non_rtti_object_vtable));
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn __non_rtti_object_dtor(_this: *mut __non_rtti_object) {
    // TRACE("(%p)\n", _this);
    bad_typeid_dtor(_this);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn __non_rtti_object_dtor(_this: *mut __non_rtti_object) {
    // TRACE("(%p)\n", _this);
    bad_typeid_dtor(_this);
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn __non_rtti_object_opequals(
    _this: *mut __non_rtti_object,
    rhs: *const __non_rtti_object,
) -> *mut __non_rtti_object {
    bad_typeid_opequals(_this as *mut bad_typeid, rhs as *const bad_typeid);
    return _this;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn __non_rtti_object_opequals(
    _this: *mut __non_rtti_object,
    rhs: *const __non_rtti_object,
) -> *mut __non_rtti_object {
    bad_typeid_opequals(_this as *mut bad_typeid, rhs as *const bad_typeid);
    return _this;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn __non_rtti_object_vector_dtor(_this: *mut __non_rtti_object, flags: u32) -> *mut core::ffi::c_void {
    if flags & 2 != 0 {
        let ptr = (_this as *mut INT_PTR).offset(-(1 as isize));
        let mut i = *ptr - 1;
        while i >= 0 {
            __non_rtti_object_dtor(_this.offset(i as isize));
            i -= 1;
        }
        operator_delete(ptr as *mut core::ffi::c_void);
    } else {
        __non_rtti_object_dtor(_this);
        if flags & 1 != 0 {
            operator_delete(_this as *mut core::ffi::c_void);
        }
    }
    _this as *mut core::ffi::c_void
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn __non_rtti_object_vector_dtor(_this: *mut __non_rtti_object, flags: u32) -> *mut core::ffi::c_void {
    if flags & 2 != 0 {
        let ptr = (_this as *mut INT_PTR).offset(-(1 as isize));
        let mut i = *ptr - 1;
        while i >= 0 {
            __non_rtti_object_dtor(_this.offset(i as isize));
            i -= 1;
        }
        operator_delete(ptr as *mut core::ffi::c_void);
    } else {
        __non_rtti_object_dtor(_this);
        if flags & 1 != 0 {
            operator_delete(_this as *mut core::ffi::c_void);
        }
    }
    _this as *mut core::ffi::c_void
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn __non_rtti_object_scalar_dtor(
    _this: *mut __non_rtti_object,
    flags: u32,
) -> *mut core::ffi::c_void {
    __non_rtti_object_dtor(_this);
    if flags & 1 != 0 {
        operator_delete((_this).cast());
    }
    _this as *mut core::ffi::c_void
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn __non_rtti_object_scalar_dtor(
    _this: *mut __non_rtti_object,
    flags: u32,
) -> *mut core::ffi::c_void {
    __non_rtti_object_dtor(_this);
    if flags & 1 != 0 {
        operator_delete((_this).cast());
    }
    _this as *mut core::ffi::c_void
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn bad_cast_ctor(_this: *mut bad_cast, name: *mut *const i8) -> *mut bad_cast {
    return __exception_ctor(_this, *name, core::ptr::addr_of!(bad_cast_vtable));
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn bad_cast_ctor(_this: *mut bad_cast, name: *mut *const i8) -> *mut bad_cast {
    return __exception_ctor(_this, *name, core::ptr::addr_of!(bad_cast_vtable));
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn bad_cast_copy_ctor(
    _this: *mut bad_cast,
    rhs: *const bad_cast,
) -> *mut bad_cast {
    return __exception_copy_ctor(_this, rhs, core::ptr::addr_of!(bad_cast_vtable));
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn bad_cast_copy_ctor(
    _this: *mut bad_cast,
    rhs: *const bad_cast,
) -> *mut bad_cast {
    return __exception_copy_ctor(_this, rhs, core::ptr::addr_of!(bad_cast_vtable));
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn bad_cast_ctor_charptr(
    _this: *mut bad_cast,
    name: *const i8
) -> *mut bad_cast {
    return __exception_ctor(_this, name, core::ptr::addr_of!(bad_cast_vtable));
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn bad_cast_ctor_charptr(
    _this: *mut bad_cast,
    name: *const i8
) -> *mut bad_cast {
    return __exception_ctor(_this, name, core::ptr::addr_of!(bad_cast_vtable));
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn bad_cast_default_ctor(_this: *mut bad_cast) -> *mut bad_cast {
    bad_cast_ctor_charptr(_this, b"bad cast\0".as_ptr() as *const i8)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn bad_cast_default_ctor(_this: *mut bad_cast) -> *mut bad_cast {
    bad_cast_ctor_charptr(_this, b"bad cast\0".as_ptr() as *const i8)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn bad_cast_dtor(_this: *mut bad_cast) {
    // TRACE("(%p)\n", _this);
    exception_dtor(_this);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn bad_cast_dtor(_this: *mut bad_cast) {
    // TRACE("(%p)\n", _this);
    exception_dtor(_this);
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn bad_cast_opequals(
    _this: *mut bad_cast,
    rhs: *const bad_cast,
) -> *mut bad_cast {
    exception_opequals(_this as *mut exception, rhs as *const exception);
    return _this;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn bad_cast_opequals(
    _this: *mut bad_cast,
    rhs: *const bad_cast,
) -> *mut bad_cast {
    exception_opequals(_this as *mut exception, rhs as *const exception);
    return _this;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn bad_cast_vector_dtor(_this: *mut bad_cast, flags: u32) -> *mut core::ffi::c_void {
    if flags & 2 != 0 {
        let ptr = (_this as *mut INT_PTR).offset(-1);
        let mut i = *ptr - 1;
        while i >= 0 {
            bad_cast_dtor(_this.offset(i as isize));
            i -= 1;
        }
        operator_delete(ptr as *mut core::ffi::c_void);
    } else {
        bad_cast_dtor(_this);
        if flags & 1 != 0 {
            operator_delete(_this as *mut core::ffi::c_void);
        }
    }
    _this as *mut core::ffi::c_void
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn bad_cast_vector_dtor(_this: *mut bad_cast, flags: u32) -> *mut core::ffi::c_void {
    if flags & 2 != 0 {
        let ptr = (_this as *mut INT_PTR).offset(-1);
        let mut i = *ptr - 1;
        while i >= 0 {
            bad_cast_dtor(_this.offset(i as isize));
            i -= 1;
        }
        operator_delete(ptr as *mut core::ffi::c_void);
    } else {
        bad_cast_dtor(_this);
        if flags & 1 != 0 {
            operator_delete(_this as *mut core::ffi::c_void);
        }
    }
    _this as *mut core::ffi::c_void
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn bad_cast_scalar_dtor(_this: *mut bad_cast, flags: u32) -> *mut core::ffi::c_void {
    bad_cast_dtor(_this);
    if flags & 1 != 0 {
        operator_delete(_this as *mut core::ffi::c_void);
    }
    _this as *mut core::ffi::c_void
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn bad_cast_scalar_dtor(_this: *mut bad_cast, flags: u32) -> *mut core::ffi::c_void {
    bad_cast_dtor(_this);
    if flags & 1 != 0 {
        operator_delete(_this as *mut core::ffi::c_void);
    }
    _this as *mut core::ffi::c_void
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn type_info_opequals_equals(_this: *mut type_info, rhs: *const type_info) -> i32 {
    let ret = (strcmp(
        ((*_this).mangled).as_ptr().offset(1),
        ((*rhs).mangled).as_ptr().offset(1)
    ) == 0) as i32;
    ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn type_info_opequals_equals(_this: *mut type_info, rhs: *const type_info) -> i32 {
    let ret = (strcmp(
        ((*_this).mangled).as_ptr().offset(1),
        ((*rhs).mangled).as_ptr().offset(1)
    ) == 0) as i32;
    ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn type_info_opnot_equals(
    _this: *mut type_info,
    rhs: *const type_info,
) -> i32 {
    let ret = if strcmp(
        (*_this).mangled.as_ptr().offset(1),
        (*rhs).mangled.as_ptr().offset(1),
    ) != 0
    {
        1
    } else {
        0
    };
    ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn type_info_opnot_equals(
    _this: *mut type_info,
    rhs: *const type_info,
) -> i32 {
    let ret = if strcmp(
        (*_this).mangled.as_ptr().offset(1),
        (*rhs).mangled.as_ptr().offset(1),
    ) != 0
    {
        1
    } else {
        0
    };
    ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn type_info_before(_this: *mut type_info, rhs: *const type_info) -> i32 {
    let ret = (strcmp((*_this).mangled.as_ptr().offset(1), (*rhs).mangled.as_ptr().offset(1)) < 0) as i32;
    ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn type_info_before(_this: *mut type_info, rhs: *const type_info) -> i32 {
    let ret = (strcmp((*_this).mangled.as_ptr().offset(1), (*rhs).mangled.as_ptr().offset(1)) < 0) as i32;
    ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn type_info_dtor(_this: *mut type_info) {
    free(((*_this).name).cast());
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn type_info_dtor(_this: *mut type_info) {
    free(((*_this).name).cast());
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn type_info_name(_this: *mut type_info) -> *const i8 {
    if (*_this).name.is_null() {
        let name = __unDName(
            core::ptr::null_mut(),
            (*_this).mangled.as_ptr().offset(1 as isize),
            0,
            // `__unDName`'s allocator hooks are NULLABLE C function pointers, so bindgen
            // types them `Option<fn>` — the C passing a bare `malloc` is the null-check
            // being decided at the call site.
            Some(malloc),
            Some(free),
            (UNDNAME_NO_ARGUMENTS | UNDNAME_32_BIT_DECODE) as u16
        );
        if !name.is_null() {
            let mut len = strlen(name);
            while len != 0 && *name.offset((len - 1) as isize) == b' ' as i8 {
                len -= 1;
                *name.offset(len as isize) = 0;
            }
            if InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*_this).name) as *mut *mut core::ffi::c_void,
                name as *mut core::ffi::c_void,
                core::ptr::null_mut()
            ) != core::ptr::null_mut() {
                free(name as *mut core::ffi::c_void);
            }
        }
    }
    (*_this).name
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn type_info_name(_this: *mut type_info) -> *const i8 {
    if (*_this).name.is_null() {
        let name = __unDName(
            core::ptr::null_mut(),
            (*_this).mangled.as_ptr().offset(1 as isize),
            0,
            // `__unDName`'s allocator hooks are NULLABLE C function pointers, so bindgen
            // types them `Option<fn>` — the C passing a bare `malloc` is the null-check
            // being decided at the call site.
            Some(malloc),
            Some(free),
            (UNDNAME_NO_ARGUMENTS | UNDNAME_32_BIT_DECODE) as u16
        );
        if !name.is_null() {
            let mut len = strlen(name);
            while len != 0 && *name.offset((len - 1) as isize) == b' ' as i8 {
                len -= 1;
                *name.offset(len as isize) = 0;
            }
            if InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*_this).name) as *mut *mut core::ffi::c_void,
                name as *mut core::ffi::c_void,
                core::ptr::null_mut()
            ) != core::ptr::null_mut() {
                free(name as *mut core::ffi::c_void);
            }
        }
    }
    (*_this).name
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "thiscall" fn type_info_raw_name(_this: *mut type_info) -> *mut i8 {
    // TRACE("(%p) returning %s\n", _this, _this->mangled);
    (*_this).mangled.as_mut_ptr()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn type_info_raw_name(_this: *mut type_info) -> *mut i8 {
    // TRACE("(%p) returning %s\n", _this, _this->mangled);
    (*_this).mangled.as_mut_ptr()
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_init_exception(base: *mut core::ffi::c_void) {
    // C: `#ifdef RTTI_USE_RVA` — i.e. NOT i386. The model wrote `target_pointer_width =
    // "64"`, which happens to select the same three arches we build and is still the wrong
    // key: cxx.h branches on `__i386__`, not on pointer width, so a 32-bit non-x86 target
    // would compile every RTTI blob in RVA form and then never patch it. Key the cfg to
    // the condition the C actually tests.
    #[cfg(not(target_arch = "x86"))]
    {
        init_type_info_rtti((base).cast());
        init_exception_rtti((base).cast());
        init_bad_typeid_rtti((base).cast());
        init_bad_cast_rtti((base).cast());
        init___non_rtti_object_rtti((base).cast());

        init_exception_cxx((base).cast());
        init_bad_typeid_cxx((base).cast());
        init_bad_cast_cxx((base).cast());
        init___non_rtti_object_cxx((base).cast());
    }
}
#[no_mangle]
pub unsafe extern "C" fn throw_exception(msg: *const i8) {
    let mut e: exception = core::mem::zeroed();
    __exception_ctor(
        core::ptr::addr_of_mut!(e),
        msg,
        core::ptr::addr_of!(exception_vtable),
    );
    _CxxThrowException(
        core::ptr::addr_of_mut!(e) as *mut core::ffi::c_void,
        (core::ptr::addr_of!(exception_exception_type)).cast(),
    );
}
#[cfg(not(feature = "ucrtsupport"))]
#[no_mangle]
pub unsafe extern "C" fn set_terminate(func: terminate_function) -> terminate_function {
    let data = msvcrt_get_thread_data();
    let previous = (*data).terminate_handler;
    (*data).terminate_handler = func;
    previous
}

#[cfg(not(feature = "ucrtsupport"))]
#[no_mangle]
pub unsafe extern "C" fn _get_terminate() -> terminate_function {
    let data: *mut thread_data_t = msvcrt_get_thread_data();
    (*data).terminate_handler
}
#[no_mangle]
pub unsafe extern "C" fn set_unexpected(func: unexpected_function) -> unexpected_function {
    let data = msvcrt_get_thread_data();
    let previous = (*data).unexpected_handler;
    (*data).unexpected_handler = func;
    return previous;
}

#[no_mangle]
pub unsafe extern "C" fn _get_unexpected() -> unexpected_function {
    let data: *mut thread_data_t = msvcrt_get_thread_data();
    (*data).unexpected_handler
}

#[no_mangle]
pub unsafe extern "C" fn _set_se_translator(func: _se_translator_function) -> _se_translator_function {
    let data = msvcrt_get_thread_data();
    let previous = (*data).se_translator;
    (*data).se_translator = func;
    previous
}

// `terminate` is msvcrt's own only when msvcrt owns the CRT. Under `__UCRTSUPPORT__` the
// genuine UCRT supplies it, so the definition drops out and the six callers in except.rs and
// except_i386.rs bind to ucrt's through the declaration below instead. Removing the
// definition WITHOUT the declaration is the trap: those callers are in bodies that stay, so
// the build would fail — loudly, which is the good case. The quiet failure is the reverse,
// leaving our definition in and silently handing ucrtbase msvcrt's terminate handler slot
// under --allow-multiple-definition.
#[cfg(feature = "ucrtsupport")]
unsafe extern "C" {
    /// `eh.h` / `corecrt_terminate.h`: `void __cdecl terminate(void);`
    pub fn terminate();
}

#[cfg(not(feature = "ucrtsupport"))]
#[no_mangle]
pub unsafe extern "C" fn terminate() {
    let data = msvcrt_get_thread_data();
    if (*data).terminate_handler.is_some() {
        // `unwrap_unchecked`, not `unwrap`: the C is a plain `if (p) p();` with no
        // failure path, and `unwrap` would add a panic branch this function does not
        // have. The `is_some` above is the null check, already done.
        (*data).terminate_handler.unwrap_unchecked()();
    }
    abort();
}

#[cfg(not(feature = "ucrtsupport"))]
#[no_mangle]
pub unsafe extern "C" fn unexpected() {
    let data = msvcrt_get_thread_data();
    if (*data).unexpected_handler.is_some() {
        // `unwrap_unchecked`, not `unwrap`: the C is a plain `if (p) p();` with no
        // failure path, and `unwrap` would add a panic branch this function does not
        // have. The `is_some` above is the null check, already done.
        (*data).unexpected_handler.unwrap_unchecked()();
    }
    terminate();
}

#[no_mangle]
pub unsafe extern "C" fn __RTtypeid(cppobj: *mut core::ffi::c_void) -> *const type_info {
    let ret: *const type_info;

    if cppobj.is_null() {
        let mut e: bad_typeid = core::mem::zeroed();
        bad_typeid_ctor(&mut e, b"Attempted a typeid of NULL pointer!\0".as_ptr() as *const i8);
        _CxxThrowException(core::ptr::addr_of_mut!(e).cast(), core::ptr::addr_of!(bad_typeid_exception_type).cast());
    }

    let obj_locator = get_obj_locator(cppobj);
    let base = get_obj_locator_base(obj_locator);
    ret = rtti_rva((*obj_locator).type_descriptor, base);

    // __EXCEPT_PAGE_FAULT. Under DUMMY_PSEH this is `if (_SEH2_Volatile0 || ...)` — always
    // false, so the handler is compiled and never runs. It arrived from the translator as
    // UNCONDITIONAL code, which made every __RTtypeid throw; see _SEH2_Volatile0 in
    // cpp_rtti.rs for why the seal pushed it that way.
    if _SEH2_Volatile0 != 0 {
        let mut e: __non_rtti_object = core::mem::zeroed();
        __non_rtti_object_ctor(&mut e, b"Bad read pointer - no RTTI data!\0".as_ptr() as *const i8);
        _CxxThrowException(core::ptr::addr_of_mut!(e).cast(), core::ptr::addr_of!(__non_rtti_object_exception_type).cast());
    }

    ret
}

#[no_mangle]
pub unsafe extern "C" fn __RTDynamicCast(
    cppobj: *mut core::ffi::c_void,
    unknown: i32,
    src: *mut type_info,
    dst: *mut type_info,
    do_throw: i32,
) -> *mut core::ffi::c_void {
    let mut ret: *mut core::ffi::c_void;

    if cppobj.is_null() {
        return core::ptr::null_mut();
    }

    let obj_locator = get_obj_locator(cppobj);
    let base = get_obj_locator_base(obj_locator);
    let obj_bases: *const rtti_object_hierarchy = rtti_rva((*obj_locator).type_hierarchy, base);
    let base_array: *const rtti_base_array = rtti_rva((*obj_bases).base_classes, base);

    // C: `if (TRACE_ON(msvcrt)) dump_obj_locator(obj_locator);` — a debug-channel trace,
    // dropped here as in every other translated msvcrt module. `dump_obj_locator` itself is
    // kept (its body is all TRACE, so it is now an empty walk) rather than deleted, so the
    // shape of the C stays visible.
    let _ = dump_obj_locator;

    ret = core::ptr::null_mut();
    let mut i = 0;
    while i < (*obj_bases).array_len {
        let base_desc: *const rtti_base_descriptor = rtti_rva((*base_array).bases[i as usize], base);
        let typ: *const type_info = rtti_rva((*base_desc).type_descriptor, base);

        if strcmp((*typ).mangled.as_ptr(), (*dst).mangled.as_ptr()) == 0 {
            let this_ptr = (cppobj as *mut u8).offset(-((*obj_locator).base_class_offset as isize)) as *mut core::ffi::c_void;
            ret = get_this_pointer(core::ptr::addr_of!((*base_desc).offsets), this_ptr);
            break;
        }
        i += 1;
    }

    if ret.is_null() && do_throw != 0 {
        let msg = b"Bad dynamic_cast!\0".as_ptr() as *const i8;
        let mut e: bad_cast = core::mem::zeroed();
        bad_cast_ctor(&mut e, &msg as *const *const i8 as *mut *const i8);
        _CxxThrowException(&mut e as *mut bad_cast as *mut core::ffi::c_void, core::ptr::addr_of!(bad_cast_exception_type).cast());
    }

    // __EXCEPT_PAGE_FAULT — never taken under DUMMY_PSEH. See __RTtypeid above.
    if _SEH2_Volatile0 != 0 {
        let mut e_nrtti: __non_rtti_object = core::mem::zeroed();
        __non_rtti_object_ctor(&mut e_nrtti, b"Access violation - no RTTI data!\0".as_ptr() as *const i8);
        _CxxThrowException(&mut e_nrtti as *mut __non_rtti_object as *mut core::ffi::c_void, core::ptr::addr_of!(__non_rtti_object_exception_type).cast());
    }

    ret
}

#[no_mangle]
pub unsafe extern "C" fn __RTCastToVoid(cppobj: *mut core::ffi::c_void) -> *mut core::ffi::c_void {
    let ret: *mut core::ffi::c_void;

    if cppobj.is_null() {
        return core::ptr::null_mut();
    }

    let obj_locator = get_obj_locator(cppobj);
    ret = (cppobj as *mut u8).offset(-((*obj_locator).base_class_offset as isize)) as *mut core::ffi::c_void;

    // __EXCEPT_PAGE_FAULT — never taken under DUMMY_PSEH. See __RTtypeid above.
    if _SEH2_Volatile0 != 0 {
        let mut e: __non_rtti_object = core::mem::zeroed();
        __non_rtti_object_ctor(&mut e, b"Access violation - no RTTI data!\0".as_ptr() as *const i8);
        _CxxThrowException(&mut e as *mut __non_rtti_object as *mut core::ffi::c_void, core::ptr::addr_of!(__non_rtti_object_exception_type).cast());
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn _CxxThrowException(object: *mut core::ffi::c_void, type_: *const cxx_exception_type) {
    let mut args: [usize; (CXX_EXCEPTION_PARAMS) as usize] = core::mem::zeroed();

    args[0] = (CXX_FRAME_MAGIC_VC6) as usize;
    args[1] = object as usize;
    args[2] = type_ as usize;
    if CXX_EXCEPTION_PARAMS == 4 {
        args[3] = rtti_rva_base(type_);
    }
    loop {
        RaiseException(CXX_EXCEPTION, EXCEPTION_NONCONTINUABLE, CXX_EXCEPTION_PARAMS, (args.as_ptr()).cast());
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * msvcrt.dll C++ objects
//  *
//  * Copyright 2000 Jon Griffiths
//  * Copyright 2003, 2004 Alexandre Julliard
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include <stdarg.h>
// #include <stdbool.h>
//
// #include "windef.h"
// #include "winternl.h"
// #include "winbase.h"
// #include "winver.h"
// #include "imagehlp.h"
// #include "wine/exception.h"
// #include "wine/debug.h"
// #include "msvcrt.h"
// #include "mtdll.h"
// #include "cppexcept.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(msvcrt);
//
// CREATE_TYPE_INFO_VTABLE
// CREATE_EXCEPTION_OBJECT(exception)
//
// struct __type_info_node
// {
//     void *memPtr;
//     struct __type_info_node* next;
// };
//
// typedef exception bad_cast;
// typedef exception bad_typeid;
// typedef exception __non_rtti_object;
//
// extern const vtable_ptr bad_typeid_vtable;
// extern const vtable_ptr bad_cast_vtable;
// extern const vtable_ptr __non_rtti_object_vtable;
// extern const vtable_ptr type_info_vtable;
//
// /* get the vtable pointer for a C++ object */
// static inline const vtable_ptr *get_vtable( void *obj )
// {
//     return *(const vtable_ptr **)obj;
// }
//
// static inline const rtti_object_locator *get_obj_locator( void *cppobj )
// {
//     const vtable_ptr *vtable = get_vtable( cppobj );
//     return (const rtti_object_locator *)vtable[-1];
// }
//
// static uintptr_t get_obj_locator_base( const rtti_object_locator *ptr )
// {
// #ifdef RTTI_USE_RVA
//     if (ptr->signature) return (uintptr_t)ptr - ptr->object_locator;
// #endif
//     return rtti_rva_base( ptr );
// }
//
// static void dump_obj_locator( const rtti_object_locator *ptr )
// {
//     int i;
//     uintptr_t base = get_obj_locator_base( ptr );
//     const rtti_object_hierarchy *h = rtti_rva( ptr->type_hierarchy, base );
//     const type_info *type_descriptor = rtti_rva( ptr->type_descriptor, base );
//
//     TRACE( "%p: sig=%08x base_offset=%08x flags=%08x type=%p %s hierarchy=%p\n",
//             ptr, ptr->signature, ptr->base_class_offset, ptr->flags,
//             type_descriptor, dbgstr_type_info(type_descriptor), h );
//     TRACE( "  hierarchy: sig=%08x attr=%08x len=%d base classes=%p\n",
//            h->signature, h->attributes, h->array_len, rtti_rva(h->base_classes, base) );
//     for (i = 0; i < h->array_len; i++)
//     {
//         const rtti_base_array *base_array = rtti_rva( h->base_classes, base );
//         const rtti_base_descriptor *bases = rtti_rva( base_array->bases[i], base );
//
//         TRACE( "    base class %p: num %d off %d,%d,%d attr %08x type %p %s\n",
//                bases, bases->num_base_classes, bases->offsets.this_offset,
//                bases->offsets.vbase_descr, bases->offsets.vbase_offset, bases->attributes,
//                rtti_rva( bases->type_descriptor, base ),
//                dbgstr_type_info((const type_info*)(base + bases->type_descriptor)) );
//     }
// }
//
// /******************************************************************
//  *		??0exception@@QAE@ABQBD@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(exception_ctor,8)
// exception *  exception_ctor(exception * _this, const char ** name)
// {
//   TRACE("(%p,%s)\n", _this, *name);
//   return __exception_ctor(_this, *name, &exception_vtable);
// }
//
// /******************************************************************
//  *		??0exception@@QAE@ABQBDH@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(exception_ctor_noalloc,12)
// exception *  exception_ctor_noalloc(exception * _this, char ** name, int noalloc)
// {
//   TRACE("(%p,%s)\n", _this, *name);
//   _this->vtable = &exception_vtable;
//   _this->name = *name;
//   _this->do_free = FALSE;
//   return _this;
// }
//
// /******************************************************************
//  *		??0exception@@QAE@XZ (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(exception_default_ctor,4)
// exception *  exception_default_ctor(exception * _this)
// {
//   TRACE("(%p)\n", _this);
//   return __exception_ctor(_this, NULL, &exception_vtable);
// }
//
// /******************************************************************
//  *		??4exception@@QAEAAV0@ABV0@@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(exception_opequals,8)
// exception *  exception_opequals(exception * _this, const exception * rhs)
// {
//   TRACE("(%p %p)\n", _this, rhs);
//   if (_this != rhs)
//   {
//       exception_dtor(_this);
//       exception_copy_ctor(_this, rhs);
//   }
//   TRACE("name = %s\n", _this->name);
//   return _this;
// }
//
// /******************************************************************
//  *		??_Gexception@@UAEPAXI@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(exception_scalar_dtor,8)
// void *  exception_scalar_dtor(exception * _this, unsigned int flags)
// {
//     TRACE("(%p %x)\n", _this, flags);
//     exception_dtor(_this);
//     if (flags & 1) operator_delete(_this);
//     return _this;
// }
//
// /******************************************************************
//  *		??0bad_typeid@@QAE@ABV0@@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(bad_typeid_copy_ctor,8)
// bad_typeid *  bad_typeid_copy_ctor(bad_typeid * _this, const bad_typeid * rhs)
// {
//   TRACE("(%p %p)\n", _this, rhs);
//   return __exception_copy_ctor(_this, rhs, &bad_typeid_vtable);
// }
//
// /******************************************************************
//  *		??0bad_typeid@@QAE@PBD@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(bad_typeid_ctor,8)
// bad_typeid *  bad_typeid_ctor(bad_typeid * _this, const char * name)
// {
//   TRACE("(%p %s)\n", _this, name);
//   return __exception_ctor(_this, name, &bad_typeid_vtable);
// }
//
// /******************************************************************
//  *		??_Fbad_typeid@@QAEXXZ (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(bad_typeid_default_ctor,4)
// bad_typeid *  bad_typeid_default_ctor(bad_typeid * _this)
// {
//   return bad_typeid_ctor( _this, "bad typeid" );
// }
//
// /******************************************************************
//  *		??1bad_typeid@@UAE@XZ (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(bad_typeid_dtor,4)
// void __thiscall bad_typeid_dtor(bad_typeid * _this)
// {
//   TRACE("(%p)\n", _this);
//   exception_dtor(_this);
// }
//
// /******************************************************************
//  *		??4bad_typeid@@QAEAAV0@ABV0@@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(bad_typeid_opequals,8)
// bad_typeid *  bad_typeid_opequals(bad_typeid * _this, const bad_typeid * rhs)
// {
//   TRACE("(%p %p)\n", _this, rhs);
//   exception_opequals(_this, rhs);
//   return _this;
// }
//
// /******************************************************************
//  *              ??_Ebad_typeid@@UAEPAXI@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(bad_typeid_vector_dtor,8)
// void *  bad_typeid_vector_dtor(bad_typeid * _this, unsigned int flags)
// {
//     TRACE("(%p %x)\n", _this, flags);
//     if (flags & 2)
//     {
//         /* we have an array, with the number of elements stored before the first object */
//         INT_PTR i, *ptr = (INT_PTR *)_this - 1;
//
//         for (i = *ptr - 1; i >= 0; i--) bad_typeid_dtor(_this + i);
//         operator_delete(ptr);
//     }
//     else
//     {
//         bad_typeid_dtor(_this);
//         if (flags & 1) operator_delete(_this);
//     }
//     return _this;
// }
//
// /******************************************************************
//  *		??_Gbad_typeid@@UAEPAXI@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(bad_typeid_scalar_dtor,8)
// void *  bad_typeid_scalar_dtor(bad_typeid * _this, unsigned int flags)
// {
//     TRACE("(%p %x)\n", _this, flags);
//     bad_typeid_dtor(_this);
//     if (flags & 1) operator_delete(_this);
//     return _this;
// }
//
// /******************************************************************
//  *		??0__non_rtti_object@@QAE@ABV0@@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(__non_rtti_object_copy_ctor,8)
// __non_rtti_object *  __non_rtti_object_copy_ctor(__non_rtti_object * _this,
//                                                                  const __non_rtti_object * rhs)
// {
//     TRACE("(%p %p)\n", _this, rhs);
//     return __exception_copy_ctor(_this, rhs, &__non_rtti_object_vtable);
// }
//
// /******************************************************************
//  *		??0__non_rtti_object@@QAE@PBD@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(__non_rtti_object_ctor,8)
// __non_rtti_object *  __non_rtti_object_ctor(__non_rtti_object * _this,
//                                                             const char * name)
// {
//   TRACE("(%p %s)\n", _this, name);
//   return __exception_ctor(_this, name, &__non_rtti_object_vtable);
// }
//
// /******************************************************************
//  *		??1__non_rtti_object@@UAE@XZ (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(__non_rtti_object_dtor,4)
// void __thiscall __non_rtti_object_dtor(__non_rtti_object * _this)
// {
//   TRACE("(%p)\n", _this);
//   bad_typeid_dtor(_this);
// }
//
// /******************************************************************
//  *		??4__non_rtti_object@@QAEAAV0@ABV0@@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(__non_rtti_object_opequals,8)
// __non_rtti_object *  __non_rtti_object_opequals(__non_rtti_object * _this,
//                                                                 const __non_rtti_object *rhs)
// {
//   TRACE("(%p %p)\n", _this, rhs);
//   bad_typeid_opequals(_this, rhs);
//   return _this;
// }
//
// /******************************************************************
//  *		??_E__non_rtti_object@@UAEPAXI@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(__non_rtti_object_vector_dtor,8)
// void *  __non_rtti_object_vector_dtor(__non_rtti_object * _this, unsigned int flags)
// {
//     TRACE("(%p %x)\n", _this, flags);
//     if (flags & 2)
//     {
//         /* we have an array, with the number of elements stored before the first object */
//         INT_PTR i, *ptr = (INT_PTR *)_this - 1;
//
//         for (i = *ptr - 1; i >= 0; i--) __non_rtti_object_dtor(_this + i);
//         operator_delete(ptr);
//     }
//     else
//     {
//         __non_rtti_object_dtor(_this);
//         if (flags & 1) operator_delete(_this);
//     }
//     return _this;
// }
//
// /******************************************************************
//  *		??_G__non_rtti_object@@UAEPAXI@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(__non_rtti_object_scalar_dtor,8)
// void *  __non_rtti_object_scalar_dtor(__non_rtti_object * _this, unsigned int flags)
// {
//   TRACE("(%p %x)\n", _this, flags);
//   __non_rtti_object_dtor(_this);
//   if (flags & 1) operator_delete(_this);
//   return _this;
// }
//
// /******************************************************************
//  *		??0bad_cast@@AAE@PBQBD@Z (MSVCRT.@)
//  *		??0bad_cast@@QAE@ABQBD@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(bad_cast_ctor,8)
// bad_cast *  bad_cast_ctor(bad_cast * _this, const char ** name)
// {
//   TRACE("(%p %s)\n", _this, *name);
//   return __exception_ctor(_this, *name, &bad_cast_vtable);
// }
//
// /******************************************************************
//  *		??0bad_cast@@QAE@ABV0@@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(bad_cast_copy_ctor,8)
// bad_cast *  bad_cast_copy_ctor(bad_cast * _this, const bad_cast * rhs)
// {
//   TRACE("(%p %p)\n", _this, rhs);
//   return __exception_copy_ctor(_this, rhs, &bad_cast_vtable);
// }
//
// /******************************************************************
//  *		??0bad_cast@@QAE@PBD@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(bad_cast_ctor_charptr,8)
// bad_cast *  bad_cast_ctor_charptr(bad_cast * _this, const char * name)
// {
//   TRACE("(%p %s)\n", _this, name);
//   return __exception_ctor(_this, name, &bad_cast_vtable);
// }
//
// /******************************************************************
//  *		??_Fbad_cast@@QAEXXZ (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(bad_cast_default_ctor,4)
// bad_cast *  bad_cast_default_ctor(bad_cast * _this)
// {
//   return bad_cast_ctor_charptr( _this, "bad cast" );
// }
//
// /******************************************************************
//  *		??1bad_cast@@UAE@XZ (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(bad_cast_dtor,4)
// void __thiscall bad_cast_dtor(bad_cast * _this)
// {
//   TRACE("(%p)\n", _this);
//   exception_dtor(_this);
// }
//
// /******************************************************************
//  *		??4bad_cast@@QAEAAV0@ABV0@@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(bad_cast_opequals,8)
// bad_cast *  bad_cast_opequals(bad_cast * _this, const bad_cast * rhs)
// {
//   TRACE("(%p %p)\n", _this, rhs);
//   exception_opequals(_this, rhs);
//   return _this;
// }
//
// /******************************************************************
//  *              ??_Ebad_cast@@UAEPAXI@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(bad_cast_vector_dtor,8)
// void *  bad_cast_vector_dtor(bad_cast * _this, unsigned int flags)
// {
//     TRACE("(%p %x)\n", _this, flags);
//     if (flags & 2)
//     {
//         /* we have an array, with the number of elements stored before the first object */
//         INT_PTR i, *ptr = (INT_PTR *)_this - 1;
//
//         for (i = *ptr - 1; i >= 0; i--) bad_cast_dtor(_this + i);
//         operator_delete(ptr);
//     }
//     else
//     {
//         bad_cast_dtor(_this);
//         if (flags & 1) operator_delete(_this);
//     }
//     return _this;
// }
//
// /******************************************************************
//  *		??_Gbad_cast@@UAEPAXI@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(bad_cast_scalar_dtor,8)
// void *  bad_cast_scalar_dtor(bad_cast * _this, unsigned int flags)
// {
//   TRACE("(%p %x)\n", _this, flags);
//   bad_cast_dtor(_this);
//   if (flags & 1) operator_delete(_this);
//   return _this;
// }
//
// /******************************************************************
//  *		??8type_info@@QBEHABV0@@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(type_info_opequals_equals,8)
// int __thiscall type_info_opequals_equals(type_info * _this, const type_info * rhs)
// {
//     int ret = !strcmp(_this->mangled + 1, rhs->mangled + 1);
//     TRACE("(%p %p) returning %d\n", _this, rhs, ret);
//     return ret;
// }
//
// /******************************************************************
//  *		??9type_info@@QBEHABV0@@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(type_info_opnot_equals,8)
// int __thiscall type_info_opnot_equals(type_info * _this, const type_info * rhs)
// {
//     int ret = !!strcmp(_this->mangled + 1, rhs->mangled + 1);
//     TRACE("(%p %p) returning %d\n", _this, rhs, ret);
//     return ret;
// }
//
// /******************************************************************
//  *		?before@type_info@@QBEHABV1@@Z (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(type_info_before,8)
// int __thiscall type_info_before(type_info * _this, const type_info * rhs)
// {
//     int ret = strcmp(_this->mangled + 1, rhs->mangled + 1) < 0;
//     TRACE("(%p %p) returning %d\n", _this, rhs, ret);
//     return ret;
// }
//
// /******************************************************************
//  *		??1type_info@@UAE@XZ (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(type_info_dtor,4)
// void __thiscall type_info_dtor(type_info * _this)
// {
//   TRACE("(%p)\n", _this);
//   free(_this->name);
// }
//
// /******************************************************************
//  *		?name@type_info@@QBEPBDXZ (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(type_info_name,4)
// const char *  type_info_name(type_info * _this)
// {
//   if (!_this->name)
//   {
//     /* Create and set the demangled name */
//     /* Note: mangled name in type_info struct always starts with a '.', while
//      * it isn't valid for mangled name.
//      * Is this '.' really part of the mangled name, or has it some other meaning ?
//      */
//     char* name = __unDName(0, _this->mangled + 1, 0,
//                            malloc, free, UNDNAME_NO_ARGUMENTS | UNDNAME_32_BIT_DECODE);
//     if (name)
//     {
//       unsigned int len = strlen(name);
//
//       /* It seems _unDName may leave blanks at the end of the demangled name */
//       while (len && name[--len] == ' ')
//         name[len] = '\0';
//
//       if (InterlockedCompareExchangePointer((void**)&_this->name, name, NULL))
//       {
//         /* Another thread set this member since we checked above - use it */
//         free(name);
//       }
//     }
//   }
//   TRACE("(%p) returning %s\n", _this, _this->name);
//   return _this->name;
// }
//
// /******************************************************************
//  *		?raw_name@type_info@@QBEPBDXZ (MSVCRT.@)
//  */
// DEFINE_THISCALL_WRAPPER(type_info_raw_name,4)
// const char *  type_info_raw_name(type_info * _this)
// {
//   TRACE("(%p) returning %s\n", _this, _this->mangled);
//   return _this->mangled;
// }
//
//
// __ASM_BLOCK_BEGIN(vtables)
//
// __ASM_VTABLE(bad_typeid,
//         VTABLE_ADD_FUNC(bad_typeid_vector_dtor)
//         VTABLE_ADD_FUNC(exception_what));
// __ASM_VTABLE(bad_cast,
//         VTABLE_ADD_FUNC(bad_cast_vector_dtor)
//         VTABLE_ADD_FUNC(exception_what));
// __ASM_VTABLE(__non_rtti_object,
//         VTABLE_ADD_FUNC(__non_rtti_object_vector_dtor)
//         VTABLE_ADD_FUNC(exception_what));
//
// __ASM_BLOCK_END
//
// DEFINE_RTTI_DATA1( bad_typeid, 0, &exception_rtti_base_descriptor, ".?AVbad_typeid@@" )
// DEFINE_RTTI_DATA1( bad_cast, 0, &exception_rtti_base_descriptor, ".?AVbad_cast@@" )
// DEFINE_RTTI_DATA2( __non_rtti_object, 0, &bad_typeid_rtti_base_descriptor, &exception_rtti_base_descriptor, ".?AV__non_rtti_object@@" )
//
// DEFINE_CXX_DATA0( exception, exception_dtor )
// DEFINE_CXX_DATA1( bad_typeid, &exception_cxx_type_info, bad_typeid_dtor )
// DEFINE_CXX_DATA1( bad_cast, &exception_cxx_type_info, bad_cast_dtor )
// DEFINE_CXX_DATA2( __non_rtti_object, &bad_typeid_cxx_type_info,
//         &exception_cxx_type_info, __non_rtti_object_dtor )
//
// void msvcrt_init_exception(void *base)
// {
// #ifdef RTTI_USE_RVA
//     init_type_info_rtti(base);
//     init_exception_rtti(base);
//     init_bad_typeid_rtti(base);
//     init_bad_cast_rtti(base);
//     init___non_rtti_object_rtti(base);
//
//     init_exception_cxx(base);
//     init_bad_typeid_cxx(base);
//     init_bad_cast_cxx(base);
//     init___non_rtti_object_cxx(base);
// #endif
// }
//
//
// void throw_exception(const char* msg)
// {
//     exception e;
//     __exception_ctor(&e, msg, &exception_vtable);
//     _CxxThrowException(&e, core::ptr::addr_of!(exception_exception_type).cast());
// }
//
// #ifndef __UCRTSUPPORT__
// /******************************************************************
//  *		?set_terminate@@YAP6AXXZP6AXXZ@Z (MSVCRT.@)
//  *
//  * Install a handler to be called when terminate() is called.
//  *
//  * PARAMS
//  *  func [I] Handler function to install
//  *
//  * RETURNS
//  *  The previously installed handler function, if any.
//  */
// terminate_function CDECL set_terminate(terminate_function func)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//     terminate_function previous = data->terminate_handler;
//     TRACE("(%p) returning %p\n",func,previous);
//     data->terminate_handler = func;
//     return previous;
// }
//
// /******************************************************************
//  *              _get_terminate (MSVCRT.@)
//  */
// terminate_function CDECL _get_terminate(void)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//     TRACE("returning %p\n", data->terminate_handler);
//     return data->terminate_handler;
// }
// #endif /* !__UCRTSUPPORT__ */
//
// /******************************************************************
//  *		?set_unexpected@@YAP6AXXZP6AXXZ@Z (MSVCRT.@)
//  *
//  * Install a handler to be called when unexpected() is called.
//  *
//  * PARAMS
//  *  func [I] Handler function to install
//  *
//  * RETURNS
//  *  The previously installed handler function, if any.
//  */
// unexpected_function CDECL set_unexpected(unexpected_function func)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//     unexpected_function previous = data->unexpected_handler;
//     TRACE("(%p) returning %p\n",func,previous);
//     data->unexpected_handler = func;
//     return previous;
// }
//
// /******************************************************************
//  *              _get_unexpected (MSVCRT.@)
//  */
// unexpected_function CDECL _get_unexpected(void)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//     TRACE("returning %p\n", data->unexpected_handler);
//     return data->unexpected_handler;
// }
//
// /******************************************************************
//  *              ?_set_se_translator@@YAP6AXIPAU_EXCEPTION_POINTERS@@@ZP6AXI0@Z@Z  (MSVCRT.@)
//  */
// _se_translator_function CDECL _set_se_translator(_se_translator_function func)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//     _se_translator_function previous = data->se_translator;
//     TRACE("(%p) returning %p\n",func,previous);
//     data->se_translator = func;
//     return previous;
// }
//
// #ifndef __UCRTSUPPORT__
// /******************************************************************
//  *		?terminate@@YAXXZ (MSVCRT.@)
//  *
//  * Default handler for an unhandled exception.
//  *
//  * PARAMS
//  *  None.
//  *
//  * RETURNS
//  *  This function does not return. Either control resumes from any
//  *  handler installed by calling set_terminate(), or (by default) abort()
//  *  is called.
//  */
// void CDECL terminate(void)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//     if (data->terminate_handler) data->terminate_handler();
//     abort();
// }
//
// /******************************************************************
//  *		?unexpected@@YAXXZ (MSVCRT.@)
//  */
// void CDECL unexpected(void)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//     if (data->unexpected_handler) data->unexpected_handler();
//     terminate();
// }
// #endif /* !__UCRTSUPPORT__ */
//
//
// /******************************************************************
//  *		__RTtypeid (MSVCRT.@)
//  *
//  * Retrieve the Run Time Type Information (RTTI) for a C++ object.
//  *
//  * PARAMS
//  *  cppobj [I] C++ object to get type information for.
//  *
//  * RETURNS
//  *  Success: A type_info object describing cppobj.
//  *  Failure: If the object to be cast has no RTTI, a __non_rtti_object
//  *           exception is thrown. If cppobj is NULL, a bad_typeid exception
//  *           is thrown. In either case, this function does not return.
//  *
//  * NOTES
//  *  This function is usually called by compiler generated code as a result
//  *  of using one of the C++ dynamic cast statements.
//  */
// const type_info*  __RTtypeid(void *cppobj)
// {
//     const type_info *ret;
//
//     if (!cppobj)
//     {
//         bad_typeid e;
//         bad_typeid_ctor( &e, "Attempted a typeid of NULL pointer!" );
//         _CxxThrowException( &e, &bad_typeid_exception_type );
//     }
//
//     __TRY
//     {
//         const rtti_object_locator *obj_locator = get_obj_locator( cppobj );
//         uintptr_t base = get_obj_locator_base( obj_locator );
//
//         ret = rtti_rva( obj_locator->type_descriptor, base );
//     }
//     __EXCEPT_PAGE_FAULT
//     {
//         __non_rtti_object e;
//         __non_rtti_object_ctor( &e, "Bad read pointer - no RTTI data!" );
//         _CxxThrowException( &e, &__non_rtti_object_exception_type );
//     }
//     __ENDTRY
//     return ret;
// }
//
// /******************************************************************
//  *		__RTDynamicCast (MSVCRT.@)
//  *
//  * Dynamically cast a C++ object to one of its base classes.
//  *
//  * PARAMS
//  *  cppobj   [I] Any C++ object to cast
//  *  unknown  [I] Reserved, set to 0
//  *  src      [I] type_info object describing cppobj
//  *  dst      [I] type_info object describing the base class to cast to
//  *  do_throw [I] TRUE = throw an exception if the cast fails, FALSE = don't
//  *
//  * RETURNS
//  *  Success: The address of cppobj, cast to the object described by dst.
//  *  Failure: NULL, If the object to be cast has no RTTI, or dst is not a
//  *           valid cast for cppobj. If do_throw is TRUE, a bad_cast exception
//  *           is thrown and this function does not return.
//  *
//  * NOTES
//  *  This function is usually called by compiler generated code as a result
//  *  of using one of the C++ dynamic cast statements.
//  */
// void*  __RTDynamicCast(void *cppobj, int unknown,
//                                    type_info *src, type_info *dst,
//                                    int do_throw)
// {
//     void *ret;
//
//     if (!cppobj) return NULL;
//
//     TRACE("obj: %p unknown: %d src: %p %s dst: %p %s do_throw: %d)\n",
//           cppobj, unknown, src, dbgstr_type_info(src), dst, dbgstr_type_info(dst), do_throw);
//
//     /* To cast an object at runtime:
//      * 1.Find out the true type of the object from the typeinfo at vtable[-1]
//      * 2.Search for the destination type in the class hierarchy
//      * 3.If destination type is found, return base object address + dest offset
//      *   Otherwise, fail the cast
//      *
//      * FIXME: the unknown parameter doesn't seem to be used for anything
//      */
//     __TRY
//     {
//         int i;
//         const rtti_object_locator *obj_locator = get_obj_locator( cppobj );
//         uintptr_t base = get_obj_locator_base( obj_locator );
//         const rtti_object_hierarchy *obj_bases = rtti_rva( obj_locator->type_hierarchy, base );
//         const rtti_base_array *base_array = rtti_rva( obj_bases->base_classes, base );
//
//         if (TRACE_ON(msvcrt)) dump_obj_locator(obj_locator);
//
//         ret = NULL;
//         for (i = 0; i < obj_bases->array_len; i++)
//         {
//             const rtti_base_descriptor *base_desc = rtti_rva( base_array->bases[i], base );
//             const type_info *typ = rtti_rva( base_desc->type_descriptor, base );
//
//             if (!strcmp(typ->mangled, dst->mangled))
//             {
//                 /* compute the correct this pointer for that base class */
//                 void *this_ptr = (char *)cppobj - obj_locator->base_class_offset;
//                 ret = get_this_pointer( &base_desc->offsets, this_ptr );
//                 break;
//             }
//         }
//         /* VC++ sets do_throw to 1 when the result of a dynamic_cast is assigned
//          * to a reference, since references cannot be NULL.
//          */
//         if (!ret && do_throw)
//         {
//             const char *msg = "Bad dynamic_cast!";
//             bad_cast e;
//             bad_cast_ctor( &e, &msg );
//             _CxxThrowException( &e, &bad_cast_exception_type );
//         }
//     }
//     __EXCEPT_PAGE_FAULT
//     {
//         __non_rtti_object e;
//         __non_rtti_object_ctor( &e, "Access violation - no RTTI data!" );
//         _CxxThrowException( &e, &__non_rtti_object_exception_type );
//     }
//     __ENDTRY
//     return ret;
// }
//
//
// /******************************************************************
//  *		__RTCastToVoid (MSVCRT.@)
//  *
//  * Dynamically cast a C++ object to a void*.
//  *
//  * PARAMS
//  *  cppobj [I] The C++ object to cast
//  *
//  * RETURNS
//  *  Success: The base address of the object as a void*.
//  *  Failure: NULL, if cppobj is NULL or has no RTTI.
//  *
//  * NOTES
//  *  This function is usually called by compiler generated code as a result
//  *  of using one of the C++ dynamic cast statements.
//  */
// void*  __RTCastToVoid(void *cppobj)
// {
//     void *ret;
//
//     if (!cppobj) return NULL;
//
//     __TRY
//     {
//         const rtti_object_locator *obj_locator = get_obj_locator( cppobj );
//         ret = (char *)cppobj - obj_locator->base_class_offset;
//     }
//     __EXCEPT_PAGE_FAULT
//     {
//         __non_rtti_object e;
//         __non_rtti_object_ctor( &e, "Access violation - no RTTI data!" );
//         _CxxThrowException( &e, &__non_rtti_object_exception_type );
//     }
//     __ENDTRY
//     return ret;
// }
//
//
// /*********************************************************************
//  *		_CxxThrowException (MSVCRT.@)
//  */
// void WINAPI _CxxThrowException( void *object, const cxx_exception_type *type )
// {
//     ULONG_PTR args[CXX_EXCEPTION_PARAMS];
//
//     args[0] = CXX_FRAME_MAGIC_VC6;
//     args[1] = (ULONG_PTR)object;
//     args[2] = (ULONG_PTR)type;
//     if (CXX_EXCEPTION_PARAMS == 4) args[3] = rtti_rva_base( type );
//     for (;;) RaiseException( CXX_EXCEPTION, EXCEPTION_NONCONTINUABLE, CXX_EXCEPTION_PARAMS, args );
// }
//
// #if _MSVCR_VER >= 80 || defined(__UCRTSUPPORT__)
//
// /*********************************************************************
//  * ?_is_exception_typeof@@YAHABVtype_info@@PAU_EXCEPTION_POINTERS@@@Z
//  * ?_is_exception_typeof@@YAHAEBVtype_info@@PEAU_EXCEPTION_POINTERS@@@Z
//  */
// int __cdecl _is_exception_typeof(const type_info *ti, EXCEPTION_POINTERS *ep)
// {
//     int ret = -1;
//
//     TRACE("(%p %p)\n", ti, ep);
//
//     __TRY
//     {
//         EXCEPTION_RECORD *rec = ep->ExceptionRecord;
//
//         if (is_cxx_exception( rec ))
//         {
//             const cxx_exception_type *et = (cxx_exception_type*)rec->ExceptionInformation[2];
//             uintptr_t base = (CXX_EXCEPTION_PARAMS == 4) ? rec->ExceptionInformation[3] : 0;
//             const cxx_type_info_table *tit = rtti_rva( et->type_info_table, base );
//             int i;
//
//             for (i=0; i<tit->count; i++)
//             {
//                 const cxx_type_info *cti = rtti_rva( tit->info[i], base );
//                 const type_info *except_ti = rtti_rva( cti->type_info, base );
//                 if (ti==except_ti || !strcmp(ti->mangled, except_ti->mangled))
//                 {
//                     ret = 1;
//                     break;
//                 }
//             }
//
//             if (i == tit->count)
//                 ret = 0;
//         }
//     }
//     __EXCEPT_PAGE_FAULT
//     __ENDTRY
//
//     if(ret == -1)
//         terminate();
//     return ret;
// }
//
// /*********************************************************************
//  * __clean_type_info_names_internal (MSVCR80.@)
//  */
// void CDECL __clean_type_info_names_internal(void *p)
// {
//     FIXME("(%p) stub\n", p);
// }
//
// /*********************************************************************
//  * ?_name_internal_method@type_info@@QBEPBDPAU__type_info_node@@@Z (MSVCR100.@)
//  */
// DEFINE_THISCALL_WRAPPER(type_info_name_internal_method,8)
// const char *  type_info_name_internal_method(type_info * _this, struct __type_info_node *node)
// {
//     static int once;
//     if (node && !once++) FIXME("type_info_node parameter ignored\n");
//
//     return type_info_name(_this);
// }
//
// void*  __AdjustPointer(void *obj, const this_ptr_offsets *off)
// {
//     return get_this_pointer(off, obj);
// }
//
// #endif
//
// #if _MSVCR_VER >= 140 || defined(__UCRTSUPPORT__)
//
// typedef struct
// {
//     char *name;
//     char mangled[1];
// } type_info140;
//
// typedef struct
// {
//     SLIST_ENTRY entry;
//     char name[1];
// } type_info_entry;
//
// static void*  type_info_entry_malloc(size_t size)
// {
//     type_info_entry *ret = malloc(FIELD_OFFSET(type_info_entry, name) + size);
//     return ret->name;
// }
//
// static void CDECL type_info_entry_free(void *ptr)
// {
//     ptr = (char*)ptr - FIELD_OFFSET(type_info_entry, name);
//     free(ptr);
// }
//
// /******************************************************************
//  *		__std_type_info_compare (UCRTBASE.@)
//  */
// int CDECL __std_type_info_compare(const type_info140 *l, const type_info140 *r)
// {
//     int ret;
//
//     if (l == r) ret = 0;
//     else ret = strcmp(l->mangled + 1, r->mangled + 1);
//     TRACE("(%p %p) returning %d\n", l, r, ret);
//     return ret;
// }
//
// /******************************************************************
//  *		__std_type_info_name (UCRTBASE.@)
//  */
// const char*  __std_type_info_name(type_info140 *ti, SLIST_HEADER *header)
// {
//       if (!ti->name)
//       {
//           char* name = __unDName(0, ti->mangled + 1, 0,
//                   type_info_entry_malloc, type_info_entry_free, UNDNAME_NO_ARGUMENTS | UNDNAME_32_BIT_DECODE);
//           if (name)
//           {
//               unsigned int len = strlen(name);
//
//               while (len && name[--len] == ' ')
//                   name[len] = '\0';
//
//               if (InterlockedCompareExchangePointer((void**)&ti->name, name, NULL))
//               {
//                   type_info_entry_free(name);
//               }
//               else
//               {
//                   type_info_entry *entry = (type_info_entry*)(name-FIELD_OFFSET(type_info_entry, name));
//                   InterlockedPushEntrySList(header, &entry->entry);
//               }
//           }
//       }
//       TRACE("(%p) returning %s\n", ti, ti->name);
//       return ti->name;
// }
//
// /******************************************************************
//  *		__std_type_info_destroy_list  (UCRTBASE.@)
//  */
// void CDECL __std_type_info_destroy_list(SLIST_HEADER *header)
// {
//     SLIST_ENTRY *cur, *next;
//
//     TRACE("(%p)\n", header);
//
//     for(cur = InterlockedFlushSList(header); cur; cur = next)
//     {
//         next = cur->Next;
//         free(cur);
//     }
// }
//
// /******************************************************************
//  *              __std_type_info_hash (UCRTBASE.@)
//  */
// size_t CDECL __std_type_info_hash(const type_info140 *ti)
// {
//     size_t hash, fnv_prime;
//     const char *p;
//
// #ifdef _WIN64
//     hash = 0xcbf29ce484222325;
//     fnv_prime = 0x100000001b3;
// #else
//     hash = 0x811c9dc5;
//     fnv_prime = 0x1000193;
// #endif
//
//     TRACE("(%p)->%s\n", ti, ti->mangled);
//
//     for(p = ti->mangled+1; *p; p++) {
//         hash ^= *p;
//         hash *= fnv_prime;
//     }
//
// #ifdef _WIN64
//     hash ^= hash >> 32;
// #endif
//
//     return hash;
// }
//
// #endif /* _MSVCR_VER >= 140 */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// cpp.c's debug channel is `msvcrt` (except.c's is `seh`), so the prefixes below differ from
// the sibling files' by exactly that. Spelled as a `macro_rules!` and not a no-op `fn`: an
// out-of-line no-op is an optimization barrier that uncloaks latent ABI mismatches into a
// `ud2`, which is a rule AGENTS.md carries because it cost a BUGCHECK.

// ── the `__UCRTSUPPORT__` half of cpp.c ──────────────────────────────────────────────────
//
// Ten bodies inside `#if _MSVCR_VER >= 80 || defined(__UCRTSUPPORT__)` and
// `#if _MSVCR_VER >= 140 || defined(__UCRTSUPPORT__)`. msvcrt compiles this TU at
// `_MSVCR_VER = 0` without the macro, so none is built there and none had a Rust body until
// ucrtbase's cohab needed them; ucrtbase.spec exports all but the two `static` helpers.
//
// rustconv translation (2026-09-05, --seal-facts --seal-calls --seal-wide, $0.12), audit
// clean -- no unverified constants, no rejects. Diffed against the C by hand afterwards,
// because sealing's blind spot is a body that keeps SOME sealed tokens and permutes the rest
// and no audit flags that. Two things that survived the diff and are worth not re-deriving:
// `__std_type_info_name`'s trailing-space trim reads `name[--len]`, whose final `len` differs
// from the Rust loop's but is dead after the loop; and `__std_type_info_hash` XORs a SIGNED
// `char`, so `*p as usize` sign-extending is the faithful reading, not a bug.

#[cfg(feature = "ucrtsupport")]
#[repr(C)]
pub struct type_info140 {
    pub name: *mut ::core::ffi::c_char,
    pub mangled: [::core::ffi::c_char; 1],
}

#[cfg(feature = "ucrtsupport")]
#[repr(C)]
pub struct type_info_entry {
    pub entry: SLIST_ENTRY,
    pub name: [::core::ffi::c_char; 1],
}

#[cfg(feature = "ucrtsupport")]
#[repr(C)]
pub struct __type_info_node {
    pub memPtr: *mut core::ffi::c_void,
    pub next: *mut __type_info_node,
}

// The C wraps this whole body in `__TRY { ... } __EXCEPT_PAGE_FAULT __ENDTRY`, and the
// handler is ABSENT here on purpose, not dropped. msvcrt builds with DUMMY_PSEH, where that
// expands to the try body followed by a branch guarded on `_SEH2_Volatile0` — an always-zero
// global — so the handler is compiled and never runs. __RTtypeid, __RTDynamicCast and
// __RTCastToVoid above carry the same reasoning at length; unlike them this one has no
// handler BODY to keep (a page fault just leaves `ret` at -1 and falls into the
// `terminate()` below), so there is nothing to guard and the branch is simply not emitted.
#[cfg(feature = "ucrtsupport")]
#[no_mangle]
pub unsafe extern "C" fn _is_exception_typeof(ti: *const type_info, ep: *mut EXCEPTION_POINTERS) -> i32 {
    let mut ret: i32 = -(1 as i32);
    let rec: *mut EXCEPTION_RECORD = (*ep).ExceptionRecord;

    if is_cxx_exception(rec) != 0 {
        let et: *const cxx_exception_type = (*rec).ExceptionInformation[2 as usize] as *const cxx_exception_type;
        // `ExceptionInformation` is `ULONG_PTR[15]`, which bindgen renders as the arch's
        // concrete width (u64 / u32). `rtti_rva` declares `base: usize`, so the cast is to the
        // CALLEE'S declared type, not to a width guessed from this arch -- the C says
        // `uintptr_t base`, which is the same thing.
        let base: usize = if (CXX_EXCEPTION_PARAMS as usize) == (4 as usize) {
            (*rec).ExceptionInformation[3 as usize] as usize
        } else {
            0 as usize
        };
        let tit: *const cxx_type_info_table = rtti_rva((*et).type_info_table, base);
        let count = (*tit).count as usize;
        let mut i: usize = 0 as usize;

        while i < count {
            let cti: *const cxx_type_info = rtti_rva((*tit).info[i], base);
            let except_ti: *const type_info = rtti_rva((*cti).type_info, base);
            // `mangled` is C's trailing flexible array, so it is `[c_char; 1]` here and needs
            // `.as_ptr()` -- the same spelling the `bad_typeid`/`bad_cast` comparisons use
            // above. NO `.offset(1)`: those skip a leading byte because `type_info_compare`
            // does, and this C is a plain `strcmp(ti->mangled, except_ti->mangled)`.
            if ti == except_ti
                || strcmp((*ti).mangled.as_ptr(), (*except_ti).mangled.as_ptr()) == 0
            {
                ret = 1 as i32;
                break;
            }
            i += 1;
        }

        if i == count {
            ret = 0 as i32;
        }
    }

    if ret == -(1 as i32) {
        terminate();
    }

    ret
}

#[cfg(feature = "ucrtsupport")]
#[no_mangle]
pub unsafe extern "C" fn __clean_type_info_names_internal(p: *mut core::ffi::c_void) {
    FIXME!("msvcrt:__clean_type_info_names_internal (%p) stub\n", p);
}
#[cfg(all(feature = "ucrtsupport", target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "thiscall" fn type_info_name_internal_method(_this: *mut type_info, node: *mut __type_info_node) -> *const i8 {
    static mut once: i32 = 0;
    if !node.is_null() && (unsafe {
        let old = once;
        once += 1;
        old == 0
    }) {
        FIXME!("msvcrt:type_info_name_internal_method type_info_node parameter ignored\n");
    }
    type_info_name(_this)
}

#[cfg(all(feature = "ucrtsupport", not(target_arch = "x86")))]
#[no_mangle]
pub unsafe extern "C" fn type_info_name_internal_method(_this: *mut type_info, node: *mut __type_info_node) -> *const i8 {
    static mut once: i32 = 0;
    if !node.is_null() && (unsafe {
        let old = once;
        once += 1;
        old == 0
    }) {
        FIXME!("msvcrt:type_info_name_internal_method type_info_node parameter ignored\n");
    }
    type_info_name(_this)
}
#[cfg(feature = "ucrtsupport")]
#[no_mangle]
pub unsafe extern "C" fn __AdjustPointer(obj: *mut core::ffi::c_void, off: *const this_ptr_offsets) -> *mut core::ffi::c_void {
    get_this_pointer(off, obj)
}
#[cfg(feature = "ucrtsupport")]
unsafe extern "C" fn type_info_entry_malloc(size: usize) -> *mut core::ffi::c_void {
    // `FIELD_OFFSET!` yields `i32`, matching the C macro's `LONG`. The C adds it to a
    // `size_t`, promoting; Rust needs the widening spelled out.
    let ret =
        malloc(FIELD_OFFSET!(type_info_entry, name) as usize + size) as *mut type_info_entry;
    return (*ret).name.as_mut_ptr() as *mut core::ffi::c_void;
}

#[cfg(feature = "ucrtsupport")]
unsafe extern "C" fn type_info_entry_free(ptr: *mut core::ffi::c_void) {
    // The C rebinds its own parameter (`ptr = (char*)ptr - FIELD_OFFSET(...)`); Rust needs the
    // binding declared `mut` to do that, and the translation emitted the assignment without it.
    let mut ptr = ptr;
    let offset = FIELD_OFFSET!(type_info_entry, name);
    ptr = (ptr as *mut u8).offset(-(offset as isize)) as *mut core::ffi::c_void;
    free(ptr);
}

#[cfg(feature = "ucrtsupport")]
#[no_mangle]
pub unsafe extern "C" fn __std_type_info_compare(
    l: *const type_info140,
    r: *const type_info140,
) -> i32 {
    let ret = if l == r {
        0
    } else {
        strcmp(
            (*l).mangled.as_ptr().add(1),
            (*r).mangled.as_ptr().add(1),
        )
    };
    ret
}

#[cfg(feature = "ucrtsupport")]
#[no_mangle]
pub unsafe extern "C" fn __std_type_info_name(
    ti: *mut type_info140,
    header: *mut SLIST_HEADER,
) -> *const core::ffi::c_char {
    if (*ti).name.is_null() {
        let name: *mut core::ffi::c_char = __unDName(
            // The C's first two literal `0`s are a NULL output buffer and a zero buflen --
            // "allocate it for me, through the two callbacks below". Only the second is an
            // integer.
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*ti).mangled)
                .cast::<core::ffi::c_char>()
                .add(1 as usize),
            0,
            // `malloc_func_t`/`free_func_t` are `Option<fn>` (C's nullable function pointer),
            // and Rust does not coerce a bare fn item into the Option.
            Some(type_info_entry_malloc),
            Some(type_info_entry_free),
            // `__unDName` declares `flags` as `unsigned short`; bindgen renders both
            // constants as u32. 8192 | 2048 = 10240, so the narrowing is exact.
            (UNDNAME_NO_ARGUMENTS | UNDNAME_32_BIT_DECODE) as u16,
        ) as *mut core::ffi::c_char;

        if !name.is_null() {
            let mut len: u32 = strlen(name) as u32;

            while len > 0 && *name.add((len - 1) as usize) as u8 == b' ' {
                len -= 1;
                *name.add(len as usize) = 0 as core::ffi::c_char;
            }

            if !InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*ti).name) as *mut *mut core::ffi::c_void,
                name as *mut core::ffi::c_void,
                core::ptr::null_mut(),
            )
            .is_null()
            {
                type_info_entry_free(name as *mut core::ffi::c_void);
            } else {
                let entry = (name as *mut u8).sub(
                    FIELD_OFFSET!(type_info_entry, name) as usize,
                ) as *mut type_info_entry;

                InterlockedPushEntrySList(
                    header,
                    core::ptr::addr_of_mut!((*entry).entry),
                );
            }
        }
    }

    (*ti).name as *const core::ffi::c_char
}

#[cfg(feature = "ucrtsupport")]
#[no_mangle]
pub unsafe extern "C" fn __std_type_info_destroy_list(header: *mut SLIST_HEADER) {
    let mut cur: *mut SLIST_ENTRY = unsafe { InterlockedFlushSList(header) };
    while !cur.is_null() {
        let next: *mut SLIST_ENTRY = unsafe { (*cur).Next };
        unsafe {
            // The C's `free(cur)` converts `SLIST_ENTRY*` to `void*` implicitly.
            free(cur as *mut core::ffi::c_void);
        }
        cur = next;
    }
}

#[cfg(feature = "ucrtsupport")]
#[no_mangle]
pub unsafe extern "C" fn __std_type_info_hash(ti: *const type_info140) -> usize {
    unsafe {
        let mut hash: usize;
        let fnv_prime: usize;

        #[cfg(target_pointer_width = "64")]
        {
            hash = 0xcbf29ce484222325;
            fnv_prime = 0x100000001b3;
        }

        #[cfg(not(target_pointer_width = "64"))]
        {
            hash = 0x811c9dc5;
            fnv_prime = 0x1000193;
        }

        let mut p = (*ti).mangled.as_ptr().add(1);

        while *p != 0 {
            hash ^= *p as usize;
            hash = hash.wrapping_mul(fnv_prime);
            p = p.add(1);
        }

        #[cfg(target_pointer_width = "64")]
        {
            hash ^= hash >> 32;
        }

        hash
    }
}
