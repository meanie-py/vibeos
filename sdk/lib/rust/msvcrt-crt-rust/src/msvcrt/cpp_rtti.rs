//! cpp.c's RTTI/vtable substrate — the part no translator can see.
//!
//! Everything here is generated in the C by MACROS (`CREATE_TYPE_INFO_VTABLE`,
//! `CREATE_EXCEPTION_OBJECT`, `DEFINE_RTTI_DATA*`, `DEFINE_CXX_DATA*`, `__ASM_VTABLE`), so
//! it never appears in cpp.c as function or data text and rustconv's scanner cannot reach
//! it. It is transcribed from the macro bodies in `dll/win32/msvcrt/cxx.h` and
//! `dll/win32/msvcrt/cppexcept.h`, and it stays a MACRO here for the reason it is one
//! there: five classes share one shape, and five hand-expanded copies would drift.
//!
//! The struct LAYOUTS are not transcribed — they come from msvcrt-sys's bindgen island, so
//! the two arms cxx.h defines under `#ifndef RTTI_USE_RVA` arrive already resolved per
//! arch. That split is the whole hazard of this file:
//!
//!   * i386  (`RTTI_USE_RVA` undefined): the descriptors hold real POINTERS, the statics
//!     are fully initialised at compile time, and there is nothing to fix up at load.
//!   * amd64/arm64 (`RTTI_USE_RVA` defined): the same fields are 32-bit RVAs off the image
//!     base. The C initialises them to `0xdeadbeef` and patches them in
//!     `msvcrt_init_exception(base)` at DLL attach. So do we.
//!
//! Same field names, different widths, different meanings — which is exactly why nothing
//! here is a hand-written Rust struct. Note the cfg key throughout is `target_arch = "x86"`
//! and NOT `target_pointer_width`: cxx.h branches on `__i386__`, and the two coincide only
//! for the three arches we happen to build.

#![allow(non_upper_case_globals)]

use crate::prelude::*;
// The translated half of the same TU: the descriptors below name its ctors and dtors as
// function pointers, exactly as DEFINE_CXX_DATA does in the C.
use super::cpp::*;
use core::ffi::{c_char, c_int, c_uint, c_void};
use msvcrt_sys::*;
// Disambiguate the two globs' `FALSE`/`TRUE` (prelude u8, island u32); every use in this
// file is a Win32 BOOL.
use msvcrt_sys::{FALSE, TRUE};

// ── vtables ──────────────────────────────────────────────────────────────────
//
// `__ASM_VTABLE(name, funcs)` lays out, in `.data`:
//
//     [ &name_rtti ]  <- one pointer BEFORE the label
//     name_vtable:    <- the exported symbol; what an object's first field points at
//     [ func0 ][ func1 ] ...
//
// The RTTI pointer at `vtable[-1]` is how `get_obj_locator` finds a type at runtime, so
// the negative index is load-bearing, not a trick — and it is why this must be assembly
// and cannot be a Rust `static` (a static gives one symbol, at its START).
//
// COFF i386 prefixes every C symbol with `_` (wine/asm.h's `__ASM_NAME`); the 64-bit
// targets do not. That decoration is the only per-arch difference in the asm, and getting
// it wrong is a link error — the one failure mode here that is loud.
macro_rules! asm_vtable {
    ($name:literal, $($func:literal),+ $(,)?) => {
        #[cfg(target_arch = "x86")]
        core::arch::global_asm!(
            ".data",
            ".balign 4",
            concat!(".long _", $name, "_rtti"),
            concat!(".globl _", $name, "_vtable"),
            concat!("_", $name, "_vtable:"),
            $(concat!(".long _", $func),)+
            ".text",
        );
        #[cfg(not(target_arch = "x86"))]
        core::arch::global_asm!(
            ".data",
            ".balign 8",
            concat!(".quad ", $name, "_rtti"),
            concat!(".globl ", $name, "_vtable"),
            concat!($name, "_vtable:"),
            $(concat!(".quad ", $func),)+
            ".text",
        );
    };
}

// The five vtables msvcrt builds at `_MSVCR_VER = 0`. `exception_old` and `bad_alloc` are
// absent on purpose: both live inside `#if _MSVCR_VER >= 80`, which this module is not.
//
// Every non-type_info vtable's second slot is `exception_what` — `what()` is inherited
// from the base class, not overridden, so all four genuinely share ONE function.
asm_vtable!("type_info", "type_info_vector_dtor");
asm_vtable!("exception", "exception_vector_dtor", "exception_what");
asm_vtable!("bad_typeid", "bad_typeid_vector_dtor", "exception_what");
asm_vtable!("bad_cast", "bad_cast_vector_dtor", "exception_what");
asm_vtable!(
    "__non_rtti_object",
    "__non_rtti_object_vector_dtor",
    "exception_what"
);

extern "C" {
    pub static type_info_vtable: vtable_ptr;
    pub static exception_vtable: vtable_ptr;
    pub static bad_typeid_vtable: vtable_ptr;
    pub static bad_cast_vtable: vtable_ptr;
    pub static __non_rtti_object_vtable: vtable_ptr;
}

// ── the RVA helpers (cxx.h) ──────────────────────────────────────────────────
//
// Generic over the pointee so ONE call shape serves both arms: on i386 the caller hands in
// a real `*const T` and gets it straight back; on 64-bit it hands in a `c_uint` RVA and the
// pointee type is inferred from the binding it is assigned to.

/// `rtti_rva(rva, base)` — i386 arm: the field already IS the pointer, `base` is unused.
#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn rtti_rva<T>(ptr: *const T, _base: usize) -> *mut T {
    ptr as *mut T
}

/// `rtti_rva(rva, base)` — RVA arm: `base + rva`, where `base` came from `rtti_rva_base`.
#[cfg(not(target_arch = "x86"))]
#[inline(always)]
pub unsafe fn rtti_rva<T>(rva: c_uint, base: usize) -> *mut T {
    base.wrapping_add(rva as usize) as *mut T
}

/// i386 has no image base to find: the descriptors hold absolute pointers, so the C
/// returns a literal 0 and every `rtti_rva` above ignores it.
#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn rtti_rva_base<T>(_ptr: *const T) -> usize {
    0
}

/// The image base of whatever module the descriptor lives in. `RtlPcToFileHeader` RETURNS
/// the base and also writes it through the out-parameter; the C reads the return value.
#[cfg(not(target_arch = "x86"))]
#[inline(always)]
pub unsafe fn rtti_rva_base<T>(ptr: *const T) -> usize {
    let mut base: *mut c_void = core::ptr::null_mut();
    RtlPcToFileHeader(ptr as *mut c_void, core::ptr::addr_of_mut!(base)) as usize
}

/// `if (descr->field)` — is a descriptor reference set? Same duality as `rtti_rva`: the
/// field is a real pointer on i386 and a 32-bit RVA elsewhere, so "unset" is `NULL` on one
/// arm and `0` on the other. The i386 arm is generic because the fields that need this are
/// not all the same Rust type — `catchblock_info.type_info` is a `*const type_info` and
/// `.handler` is an `Option<fn>` — but every one of them is pointer-sized, which is what
/// makes the reinterpretation sound.
#[cfg(target_arch = "x86")]
#[inline(always)]
pub fn rtti_is_set<T: Copy>(v: T) -> bool {
    const { assert!(core::mem::size_of::<T>() == core::mem::size_of::<usize>()) };
    unsafe { core::mem::transmute_copy::<T, usize>(&v) != 0 }
}

#[cfg(not(target_arch = "x86"))]
#[inline(always)]
pub fn rtti_is_set(rva: c_uint) -> bool {
    rva != 0
}

/// A descriptor's `handler` field as a plain code address. `rtti_rva` cannot serve here on
/// i386: the field is already a *function* pointer (`void *(*)(void)`), not a `*const T`,
/// so there is nothing to add a base to and the conversion is a reinterpretation.
#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn rtti_handler<T: Copy>(h: T, _base: usize) -> *mut c_void {
    const { assert!(core::mem::size_of::<T>() == core::mem::size_of::<usize>()) };
    core::mem::transmute_copy::<T, *mut c_void>(&h)
}

#[cfg(not(target_arch = "x86"))]
#[inline(always)]
pub unsafe fn rtti_handler(rva: c_uint, base: usize) -> *mut c_void {
    base.wrapping_add(rva as usize) as *mut c_void
}

/// cppexcept.h's `get_this_pointer`. The virtual-base hop runs only when `vbase_descr >= 0`
/// — every descriptor this file builds sets it to -1, so the hop is dead for msvcrt's own
/// classes and live for a foreign C++ object walking in through `__RTDynamicCast`.
#[inline]
pub unsafe fn get_this_pointer(off: *const this_ptr_offsets, object: *mut c_void) -> *mut c_void {
    if object.is_null() {
        return core::ptr::null_mut();
    }
    let mut object = object as *mut c_char;
    if (*off).vbase_descr >= 0 {
        object = object.offset((*off).vbase_descr as isize);
        let offset_ptr = (*(object as *mut *mut c_char)).offset((*off).vbase_offset as isize)
            as *const c_int;
        object = object.offset(*offset_ptr as isize);
    }
    object.offset((*off).this_offset as isize) as *mut c_void
}

// ── static storage ───────────────────────────────────────────────────────────

/// C file-scope data, mutable on the RVA arm because `msvcrt_init_exception` patches it,
/// and mutable on BOTH arms for the `type_info` blobs (`type_info_name` fills `.name`
/// lazily). Rust wants `Sync` for a `static` and none of these are; they are written at
/// DLL attach from one thread and read-only afterwards, exactly as in the C. Saying that
/// once here beats scattering `static mut` — whose references are a hard error from
/// Rust 2024 — across forty items.
#[repr(transparent)]
pub struct SyncCell<T>(pub core::cell::UnsafeCell<T>);
unsafe impl<T> Sync for SyncCell<T> {}
impl<T> SyncCell<T> {
    #[inline(always)]
    pub const fn new(v: T) -> Self {
        Self(core::cell::UnsafeCell::new(v))
    }
    #[inline(always)]
    pub const fn get(&self) -> *mut T {
        self.0.get()
    }
}

/// The C writes `0xdeadbeef` into every RVA slot it is going to patch, so an unpatched read
/// is a wild address rather than a plausible one. Keep the poison: it is the only thing
/// between "init never ran" and "init ran and produced offset 0".
#[cfg(not(target_arch = "x86"))]
const DEADBEEF: c_uint = 0xdead_beef;

/// `type_info.mangled` is declared `char[128]` and is genuinely variable-length in the real
/// compiler-emitted RTTI; for our own static descriptors the tail is just NUL padding.
const fn mangled128(s: &[u8]) -> [c_char; 128] {
    let mut out = [0 as c_char; 128];
    let mut i = 0;
    while i < s.len() {
        out[i] = s[i] as c_char;
        i += 1;
    }
    out
}

/// `(char *)&x - base`, stored into an `unsigned int`. The truncation to 32 bits is the
/// POINT (that is what an RVA is), so it is a wrapping subtract and a narrowing cast, not
/// a checked one.
#[cfg(not(target_arch = "x86"))]
#[inline(always)]
unsafe fn rva_of<T>(p: *const T, base: *mut c_char) -> c_uint {
    (p as usize).wrapping_sub(base as usize) as c_uint
}

// `this_ptr_offsets{ 0, -1, 0 }` — every descriptor and every cxx_type_info in this file
// uses it, so it is named once. `-1` in `vbase_descr` is what disables the virtual-base
// hop in `get_this_pointer`.
const NO_VBASE: this_ptr_offsets = this_ptr_offsets {
    this_offset: 0,
    vbase_descr: -1,
    vbase_offset: 0,
};

// ── DEFINE_RTTI_DATA (cxx.h) ─────────────────────────────────────────────────
//
// One invocation per class, expanding to both arms. `bases` lists the OTHER classes'
// `_rtti_base_descriptor`s, in the order the C passes them to DEFINE_RTTI_DATA1/2 — that
// order is the class hierarchy and `__RTDynamicCast` walks it linearly.
macro_rules! define_rtti_data {
    (
        type_info: $ti:ident,
        base_descr: $bd:ident,
        base_array: $ba:ident,
        hierarchy: $h:ident,
        locator: $loc:ident,
        init: $init:ident,
        nbases: $nb:expr,
        mangled: $mangled:expr,
        bases: [$($cl:ident),* $(,)?] $(,)?
    ) => {
        pub static $ti: SyncCell<type_info> = SyncCell::new(type_info {
            vtable: core::ptr::addr_of!(type_info_vtable),
            name: core::ptr::null_mut(),
            mangled: mangled128($mangled),
        });

        // ---- i386: absolute pointers, fully initialised at compile time ----
        #[cfg(target_arch = "x86")]
        pub static $bd: SyncCell<rtti_base_descriptor> = SyncCell::new(rtti_base_descriptor {
            type_descriptor: core::ptr::addr_of!($ti) as *const type_info,
            num_base_classes: $nb,
            offsets: NO_VBASE,
            attributes: 64,
        });
        #[cfg(target_arch = "x86")]
        pub static $ba: SyncCell<rtti_base_array> = SyncCell::new(rtti_base_array {
            bases: {
                // `bases[0]` is the class ITSELF, then the declared bases, then NULL.
                let mut b = [core::ptr::null::<rtti_base_descriptor>(); 10];
                b[0] = core::ptr::addr_of!($bd) as *const rtti_base_descriptor;
                let mut _i = 1;
                $(
                    b[_i] = core::ptr::addr_of!($cl) as *const rtti_base_descriptor;
                    _i += 1;
                )*
                b
            },
        });
        #[cfg(target_arch = "x86")]
        pub static $h: SyncCell<rtti_object_hierarchy> = SyncCell::new(rtti_object_hierarchy {
            signature: 0,
            attributes: 0,
            array_len: $nb + 1,
            base_classes: core::ptr::addr_of!($ba) as *const rtti_base_array,
        });
        // NOT `static`: the vtable assembly above references this symbol by name.
        #[cfg(target_arch = "x86")]
        #[no_mangle]
        pub static $loc: SyncCell<rtti_object_locator> = SyncCell::new(rtti_object_locator {
            signature: 0,
            base_class_offset: 0,
            flags: 0,
            type_descriptor: core::ptr::addr_of!($ti) as *const type_info,
            type_hierarchy: core::ptr::addr_of!($h) as *const rtti_object_hierarchy,
        });
        /// i386 has nothing to patch — `msvcrt_init_exception`'s whole body is inside
        /// `#ifdef RTTI_USE_RVA`, so this arm does not exist in the C at all.
        #[cfg(target_arch = "x86")]
        #[inline(always)]
        pub unsafe fn $init(_base: *mut c_char) {}

        // ---- amd64/arm64: RVAs, poisoned until msvcrt_init_exception runs ----
        #[cfg(not(target_arch = "x86"))]
        pub static $bd: SyncCell<rtti_base_descriptor> = SyncCell::new(rtti_base_descriptor {
            type_descriptor: DEADBEEF,
            num_base_classes: $nb,
            offsets: NO_VBASE,
            attributes: 64,
        });
        #[cfg(not(target_arch = "x86"))]
        pub static $ba: SyncCell<rtti_base_array> = SyncCell::new(rtti_base_array {
            bases: [DEADBEEF; 10],
        });
        #[cfg(not(target_arch = "x86"))]
        pub static $h: SyncCell<rtti_object_hierarchy> = SyncCell::new(rtti_object_hierarchy {
            signature: 0,
            attributes: 0,
            array_len: $nb + 1,
            base_classes: DEADBEEF,
        });
        // `signature` is 1 here and 0 on i386, and it is not decoration: it is what
        // `get_obj_locator_base` tests to decide whether `object_locator` holds a usable
        // self-RVA. The sixth field exists only in this arm.
        #[cfg(not(target_arch = "x86"))]
        #[no_mangle]
        pub static $loc: SyncCell<rtti_object_locator> = SyncCell::new(rtti_object_locator {
            signature: 1,
            base_class_offset: 0,
            flags: 0,
            type_descriptor: DEADBEEF,
            type_hierarchy: DEADBEEF,
            object_locator: DEADBEEF,
        });
        #[cfg(not(target_arch = "x86"))]
        pub unsafe fn $init(base: *mut c_char) {
            (*$bd.get()).type_descriptor = rva_of($ti.get(), base);
            (*$ba.get()).bases[0] = rva_of($bd.get(), base);
            #[allow(unused_mut, unused_assignments)]
            let mut i = 1usize;
            $(
                (*$ba.get()).bases[i] = rva_of($cl.get(), base);
                i += 1;
            )*
            // The C writes every remaining slot too, as `(char *)NULL - base` — a garbage
            // RVA that is never read because `array_len` bounds the walk. Reproduced rather
            // than tidied: leaving them poisoned would diverge, and "never read" is a
            // property of the walk, not of the data.
            while i < 10 {
                (*$ba.get()).bases[i] = rva_of(core::ptr::null::<c_char>(), base);
                i += 1;
            }
            (*$h.get()).base_classes = rva_of($ba.get(), base);
            (*$loc.get()).type_descriptor = rva_of($ti.get(), base);
            (*$loc.get()).type_hierarchy = rva_of($h.get(), base);
            (*$loc.get()).object_locator = rva_of($loc.get(), base);
        }
    };
}

// `_MSVCR_VER = 0`, so the mangled names carry NO `@std` — cppexcept.h's
// EXCEPTION_MANGLED_NAME picks `.?AVexception@@` over `.?AVexception@std@@` below 80, and
// cpp.c's `#else` arm does the same for the three derived classes. These strings are
// compared with `strcmp` by every dynamic_cast in every C++ program in the system, so the
// `@std` variant is not a cosmetic difference.
define_rtti_data! {
    type_info: type_info_type_info, base_descr: type_info_rtti_base_descriptor,
    base_array: type_info_rtti_base_array, hierarchy: type_info_hierarchy,
    locator: type_info_rtti, init: init_type_info_rtti,
    nbases: 0, mangled: b".?AVtype_info@@", bases: [],
}
define_rtti_data! {
    type_info: exception_type_info, base_descr: exception_rtti_base_descriptor,
    base_array: exception_rtti_base_array, hierarchy: exception_hierarchy,
    locator: exception_rtti, init: init_exception_rtti,
    nbases: 0, mangled: b".?AVexception@@", bases: [],
}
define_rtti_data! {
    type_info: bad_typeid_type_info, base_descr: bad_typeid_rtti_base_descriptor,
    base_array: bad_typeid_rtti_base_array, hierarchy: bad_typeid_hierarchy,
    locator: bad_typeid_rtti, init: init_bad_typeid_rtti,
    nbases: 1, mangled: b".?AVbad_typeid@@",
    bases: [exception_rtti_base_descriptor],
}
define_rtti_data! {
    type_info: bad_cast_type_info, base_descr: bad_cast_rtti_base_descriptor,
    base_array: bad_cast_rtti_base_array, hierarchy: bad_cast_hierarchy,
    locator: bad_cast_rtti, init: init_bad_cast_rtti,
    nbases: 1, mangled: b".?AVbad_cast@@",
    bases: [exception_rtti_base_descriptor],
}
define_rtti_data! {
    type_info: __non_rtti_object_type_info,
    base_descr: __non_rtti_object_rtti_base_descriptor,
    base_array: __non_rtti_object_rtti_base_array,
    hierarchy: __non_rtti_object_hierarchy,
    locator: __non_rtti_object_rtti, init: init___non_rtti_object_rtti,
    nbases: 2, mangled: b".?AV__non_rtti_object@@",
    bases: [bad_typeid_rtti_base_descriptor, exception_rtti_base_descriptor],
}

// ── DEFINE_CXX_DATA (cxx.h) ──────────────────────────────────────────────────
//
// The throw descriptor: what `_CxxThrowException` hands the exception dispatcher so a
// `catch` can decide whether this object matches, and how to copy it into the handler's
// frame. `type_info` has none — it is not a throwable class.
macro_rules! define_cxx_data {
    (
        type_info: $ti:ident,
        cxx_type_info: $cti:ident,
        cxx_type_table: $ctt:ident,
        exception_type: $et:ident,
        init: $init:ident,
        nbases: $nb:expr,
        copy_ctor: $copy:ident,
        dtor: $dtor:ident,
        bases: [$($cl:ident),* $(,)?] $(,)?
    ) => {
        // ---- i386 ----
        #[cfg(target_arch = "x86")]
        pub static $cti: SyncCell<cxx_type_info> = SyncCell::new(cxx_type_info {
            flags: 0,
            type_info: core::ptr::addr_of!($ti) as *const type_info,
            offsets: NO_VBASE,
            size: core::mem::size_of::<exception>() as c_uint,
            copy_ctor: $copy as *const () as *mut c_void,
        });
        #[cfg(target_arch = "x86")]
        pub static $ctt: SyncCell<cxx_type_info_table> = SyncCell::new(cxx_type_info_table {
            count: $nb + 1,
            info: {
                let mut t = [core::ptr::null::<cxx_type_info>(); 5];
                t[0] = core::ptr::addr_of!($cti) as *const cxx_type_info;
                let mut _i = 1;
                $(
                    t[_i] = core::ptr::addr_of!($cl) as *const cxx_type_info;
                    _i += 1;
                )*
                t
            },
        });
        #[cfg(target_arch = "x86")]
        pub static $et: SyncCell<cxx_exception_type> = SyncCell::new(cxx_exception_type {
            flags: 0,
            destructor: $dtor as *const () as *mut c_void,
            custom_handler: core::ptr::null_mut(),
            type_info_table: core::ptr::addr_of!($ctt) as *const cxx_type_info_table,
        });
        #[cfg(target_arch = "x86")]
        #[inline(always)]
        pub unsafe fn $init(_base: *mut c_char) {}

        // ---- amd64/arm64 ----
        #[cfg(not(target_arch = "x86"))]
        pub static $cti: SyncCell<cxx_type_info> = SyncCell::new(cxx_type_info {
            flags: 0,
            type_info: DEADBEEF,
            offsets: NO_VBASE,
            size: core::mem::size_of::<exception>() as c_uint,
            copy_ctor: DEADBEEF,
        });
        #[cfg(not(target_arch = "x86"))]
        pub static $ctt: SyncCell<cxx_type_info_table> = SyncCell::new(cxx_type_info_table {
            count: $nb + 1,
            info: [DEADBEEF; 5],
        });
        // `custom_handler` is a plain 0 in BOTH arms, not DEADBEEF: the C initialises it to
        // `0` and never patches it, and the dispatcher tests it for null.
        #[cfg(not(target_arch = "x86"))]
        pub static $et: SyncCell<cxx_exception_type> = SyncCell::new(cxx_exception_type {
            flags: 0,
            destructor: DEADBEEF,
            custom_handler: 0,
            type_info_table: DEADBEEF,
        });
        #[cfg(not(target_arch = "x86"))]
        pub unsafe fn $init(base: *mut c_char) {
            (*$cti.get()).type_info = rva_of($ti.get(), base);
            (*$cti.get()).copy_ctor = rva_of($copy as *const (), base);
            (*$ctt.get()).info[0] = rva_of($cti.get(), base);
            #[allow(unused_mut, unused_assignments)]
            let mut i = 1usize;
            $(
                (*$ctt.get()).info[i] = rva_of($cl.get(), base);
                i += 1;
            )*
            let _ = i; // the C leaves the remaining `info[]` slots at their initial value
            (*$et.get()).destructor = rva_of($dtor as *const (), base);
            (*$et.get()).type_info_table = rva_of($ctt.get(), base);
        }
    };
}

define_cxx_data! {
    type_info: exception_type_info, cxx_type_info: exception_cxx_type_info,
    cxx_type_table: exception_cxx_type_table, exception_type: exception_exception_type,
    init: init_exception_cxx, nbases: 0,
    copy_ctor: exception_copy_ctor, dtor: exception_dtor, bases: [],
}
define_cxx_data! {
    type_info: bad_typeid_type_info, cxx_type_info: bad_typeid_cxx_type_info,
    cxx_type_table: bad_typeid_cxx_type_table, exception_type: bad_typeid_exception_type,
    init: init_bad_typeid_cxx, nbases: 1,
    copy_ctor: bad_typeid_copy_ctor, dtor: bad_typeid_dtor,
    bases: [exception_cxx_type_info],
}
define_cxx_data! {
    type_info: bad_cast_type_info, cxx_type_info: bad_cast_cxx_type_info,
    cxx_type_table: bad_cast_cxx_type_table, exception_type: bad_cast_exception_type,
    init: init_bad_cast_cxx, nbases: 1,
    copy_ctor: bad_cast_copy_ctor, dtor: bad_cast_dtor,
    bases: [exception_cxx_type_info],
}
define_cxx_data! {
    type_info: __non_rtti_object_type_info,
    cxx_type_info: __non_rtti_object_cxx_type_info,
    cxx_type_table: __non_rtti_object_cxx_type_table,
    exception_type: __non_rtti_object_exception_type,
    init: init___non_rtti_object_cxx, nbases: 2,
    copy_ctor: __non_rtti_object_copy_ctor, dtor: __non_rtti_object_dtor,
    bases: [bad_typeid_cxx_type_info, exception_cxx_type_info],
}

// ── the macro-generated FUNCTIONS ────────────────────────────────────────────
//
// `CREATE_EXCEPTION_OBJECT(exception)` and `CREATE_TYPE_INFO_VTABLE` expand to these
// seven bodies. They are ordinary cpp.c exports — they are here rather than in cpp.rs only
// because the scanner sees a macro invocation where their text would be.
//
// The `__thiscall` ones carry the same per-arch dual rustconv emits for the rest of the
// file: `this` in ECX on i386, first stack argument everywhere else.

/// `CREATE_EXCEPTION_OBJECT`'s shared ctor. Note it COPIES the string and sets `do_free`,
/// which is what makes `exception::name` safe to free in the dtor; the `noalloc` entry
/// point in cpp.rs is the one that deliberately does not.
pub unsafe fn __exception_ctor(
    this: *mut exception,
    str_: *const c_char,
    vtbl: *const vtable_ptr,
) -> *mut exception {
    if !str_.is_null() {
        let len = strlen(str_) + 1;
        (*this).name = malloc(len) as *mut c_char;
        memcpy((*this).name as *mut c_void, str_ as *const c_void, len);
        (*this).do_free = 1;
    } else {
        (*this).name = core::ptr::null_mut();
        (*this).do_free = 0;
    }
    (*this).vtable = vtbl;
    this
}

/// The copy ctor takes the STRING path only when the source owns its string; otherwise it
/// copies the struct wholesale and re-points the vtable, deliberately aliasing `name`
/// without taking ownership (`do_free` rides along as 0).
pub unsafe fn __exception_copy_ctor(
    this: *mut exception,
    rhs: *const exception,
    vtbl: *const vtable_ptr,
) -> *mut exception {
    if (*rhs).do_free != 0 {
        __exception_ctor(this, (*rhs).name, vtbl);
    } else {
        *this = *rhs;
        (*this).vtable = vtbl;
    }
    this
}

macro_rules! thiscall_fn {
    (
        $(#[$meta:meta])*
        fn $name:ident ( $($p:ident : $t:ty),* $(,)? ) $(-> $r:ty)? $body:block
    ) => {
        $(#[$meta])*
        #[cfg(target_arch = "x86")]
        #[no_mangle]
        pub unsafe extern "thiscall" fn $name($($p: $t),*) $(-> $r)? $body

        $(#[$meta])*
        #[cfg(not(target_arch = "x86"))]
        #[no_mangle]
        pub unsafe extern "C" fn $name($($p: $t),*) $(-> $r)? $body
    };
}

thiscall_fn! {
    /// `??0exception@@QAE@ABV0@@Z`
    fn exception_copy_ctor(this: *mut exception, rhs: *const exception) -> *mut exception {
        __exception_copy_ctor(this, rhs, core::ptr::addr_of!(exception_vtable))
    }
}

thiscall_fn! {
    /// `??1exception@@UAE@XZ`. Frees only what the object OWNS — see __exception_copy_ctor.
    fn exception_dtor(this: *mut exception) {
        if (*this).do_free != 0 {
            free((*this).name as *mut c_void);
        }
    }
}

thiscall_fn! {
    /// `??_Eexception@@UAEPAXI@Z`. `flags & 2` means "array": the element count sits in the
    /// word BEFORE the first object, and the whole allocation starts there — which is why
    /// the delete is of `ptr`, not of `this`. Destruction runs backwards, as C++ requires.
    fn exception_vector_dtor(this: *mut exception, flags: c_uint) -> *mut c_void {
        if flags & 2 != 0 {
            let ptr = (this as *mut isize).offset(-1);
            let mut i = *ptr - 1;
            while i >= 0 {
                exception_dtor(this.offset(i as isize));
                i -= 1;
            }
            operator_delete(ptr as *mut c_void);
        } else {
            exception_dtor(this);
            if flags & 1 != 0 {
                operator_delete(this as *mut c_void);
            }
        }
        this as *mut c_void
    }
}

thiscall_fn! {
    /// `?what@exception@@UBEPBDXZ`. Shared by all four exception vtables.
    fn exception_what(this: *mut exception) -> *const c_char {
        if !(*this).name.is_null() {
            (*this).name
        } else {
            c"Unknown exception".as_ptr()
        }
    }
}

thiscall_fn! {
    /// `??_Etype_info@@UAEPAXI@Z` (CREATE_TYPE_INFO_VTABLE). Same array convention as
    /// `exception_vector_dtor`, but it frees the lazily-demangled `name` rather than
    /// calling a dtor, and in the array case it does NOT free the objects themselves —
    /// only `ptr`, the head of the allocation.
    fn type_info_vector_dtor(this: *mut type_info, flags: c_uint) -> *mut c_void {
        if flags & 2 != 0 {
            let ptr = (this as *mut isize).offset(-1);
            let mut i = *ptr - 1;
            while i >= 0 {
                free((*this.offset(i as isize)).name as *mut c_void);
                i -= 1;
            }
            free(ptr as *mut c_void);
        } else {
            free((*this).name as *mut c_void);
            if flags & 1 != 0 {
                free(this as *mut c_void);
            }
        }
        this as *mut c_void
    }
}

// ── cppexcept.h's `static inline` helpers ────────────────────────────────────
//
// These five are `static inline` in cppexcept.h, so they have NO SYMBOL in any object and
// rustconv — which reads .c files — never sees them. cpp.c and except.c both include the
// header and both get their own inlined copy; here they are written once and shared, which
// is the same arrangement `get_this_pointer` above already has.
//
// `call_dtor`/`call_copy_ctor` are the exception: on i386 the header declares them EXTERN
// and they are real asm thunks in dll/win32/msvcrt/i386/exception_ptr.s, because a
// `__thiscall` call through a function POINTER cannot be spelled in C there. Everywhere
// else __thiscall is the ordinary convention and the header inlines the cast. That split is
// why these are a cfg pair and not one body.

/// cppexcept.h's `is_cxx_exception`. All three tests matter and the middle one is the
/// interesting one: `CXX_EXCEPTION_PARAMS` is 3 on i386 and 4 on 64-bit, so a record
/// carrying the wrong count is not ours even though its code says it is.
#[inline]
pub unsafe fn is_cxx_exception(rec: *mut EXCEPTION_RECORD) -> BOOL {
    if (*rec).ExceptionCode != CXX_EXCEPTION {
        return FALSE as BOOL;
    }
    if (*rec).NumberParameters != CXX_EXCEPTION_PARAMS {
        return FALSE as BOOL;
    }
    // ExceptionInformation is ULONG_PTR: u32 on i386, u64 elsewhere. Widen the magic to
    // whatever that is rather than narrowing the parameter.
    let magic = (*rec).ExceptionInformation[0];
    (magic >= CXX_FRAME_MAGIC_VC6 as _ && magic <= CXX_FRAME_MAGIC_VC8 as _) as BOOL
}

/// cppexcept.h's `call_dtor` — i386 arm: the real `__thiscall` thunk in
/// `i386/exception_ptr.s`, reached as an extern exactly as the C reaches it.
#[cfg(target_arch = "x86")]
extern "C" {
    pub fn call_dtor(func: *mut c_void, this: *mut c_void);
    pub fn call_copy_ctor(func: *mut c_void, this: *mut c_void, src: *mut c_void, has_vbase: c_int);
}

/// cppexcept.h's `call_dtor` — non-i386 arm: `__thiscall` IS the default convention, so
/// the header just casts the pointer and calls it. So do we.
#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn call_dtor(func: *mut c_void, this: *mut c_void) {
    let f: unsafe extern "C" fn(*mut c_void) = core::mem::transmute(func);
    f(this);
}

#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn call_copy_ctor(
    func: *mut c_void,
    this: *mut c_void,
    src: *mut c_void,
    has_vbase: c_int,
) {
    if has_vbase != 0 {
        let f: unsafe extern "C" fn(*mut c_void, *mut c_void, BOOL) = core::mem::transmute(func);
        f(this, src, 1);
    } else {
        let f: unsafe extern "C" fn(*mut c_void, *mut c_void) = core::mem::transmute(func);
        f(this, src);
    }
}

/// cppexcept.h's `find_caught_type` — does this catch block catch this thrown type?
///
/// A null `catch_ti` is `catch(...)` and matches the first entry outright. Otherwise the
/// match is by MANGLED NAME rather than by pointer, because the thrown type and the caught
/// type are two different static descriptors when the throw and the catch are in different
/// images; the pointer compare is only a fast path in front of the `strcmp`.
///
/// The two flag tests are one-directional on purpose, and inverting either would be silent:
/// a `const`-qualified exception may NOT be caught by a non-const handler, but a plain
/// exception is perfectly catchable by a `const` one.
#[inline]
pub unsafe fn find_caught_type(
    exc_type: *mut cxx_exception_type,
    base: usize,
    catch_ti: *const type_info,
    catch_flags: UINT,
) -> *const cxx_type_info {
    let type_info_table: *const cxx_type_info_table = rtti_rva((*exc_type).type_info_table, base);
    let mut i: UINT = 0;
    while i < (*type_info_table).count {
        let ty: *const cxx_type_info = rtti_rva((*type_info_table).info[i as usize], base);
        let ti: *const type_info = rtti_rva((*ty).type_info, base);

        if catch_ti.is_null() {
            return ty; /* catch(...) matches any type */
        }
        if catch_ti != ti
            && strcmp(
                core::ptr::addr_of!((*catch_ti).mangled) as *const c_char,
                core::ptr::addr_of!((*ti).mangled) as *const c_char,
            ) != 0
        {
            i += 1;
            continue;
        }
        /* type is the same, now check the flags */
        if ((*exc_type).flags & TYPE_FLAG_CONST) != 0 && (catch_flags & TYPE_FLAG_CONST) == 0 {
            i += 1;
            continue;
        }
        if ((*exc_type).flags & TYPE_FLAG_VOLATILE) != 0 && (catch_flags & TYPE_FLAG_VOLATILE) == 0
        {
            i += 1;
            continue;
        }
        return ty; /* it matched */
    }
    core::ptr::null()
}

/// cppexcept.h's `copy_exception` — put the thrown object where the catch block wants it.
///
/// Three genuinely different copies, chosen by flag bits, which is why those bits are
/// derived from the header rather than transcribed:
///   * `TYPE_FLAG_REFERENCE` — bind, do not copy. `*dest` becomes the object itself.
///   * `CLASS_IS_SIMPLE_TYPE` — copy the bytes. The pointer-sized case is re-adjusted
///     afterwards because a copied POINTER still needs its `this` fixed up.
///   * otherwise — run the copy constructor, or `memmove` if the class has none.
#[inline]
pub unsafe fn copy_exception(
    object: *mut c_void,
    dest: *mut *mut c_void,
    catch_flags: UINT,
    ty: *const cxx_type_info,
    base: usize,
) {
    if (catch_flags & TYPE_FLAG_REFERENCE) != 0 {
        *dest = get_this_pointer(core::ptr::addr_of!((*ty).offsets), object);
    } else if ((*ty).flags & CLASS_IS_SIMPLE_TYPE) != 0 {
        memmove(dest as *mut c_void, object, (*ty).size as usize);
        /* if it is a pointer, adjust it */
        if (*ty).size as usize == core::mem::size_of::<*mut c_void>() {
            *dest = get_this_pointer(core::ptr::addr_of!((*ty).offsets), *dest);
        }
    } else if !rtti_is_null_ctor((*ty).copy_ctor) {
        call_copy_ctor(
            rtti_rva((*ty).copy_ctor, base),
            dest as *mut c_void,
            get_this_pointer(core::ptr::addr_of!((*ty).offsets), object),
            ((*ty).flags & CLASS_HAS_VIRTUAL_BASE_CLASS) as c_int,
        );
    } else {
        memmove(
            dest as *mut c_void,
            get_this_pointer(core::ptr::addr_of!((*ty).offsets), object),
            (*ty).size as usize,
        );
    }
}

/// `if (type->copy_ctor)` — one test, two field types. `copy_ctor` is a real pointer on
/// i386 and a 32-bit RVA elsewhere, and "is it set" is spelled differently for each. Same
/// shape as `rtti_rva` above, and it exists so `copy_exception` can stay one body.
#[cfg(target_arch = "x86")]
#[inline(always)]
fn rtti_is_null_ctor(p: *mut c_void) -> bool {
    p.is_null()
}

#[cfg(not(target_arch = "x86"))]
#[inline(always)]
fn rtti_is_null_ctor(rva: c_uint) -> bool {
    rva == 0
}

// ── DUMMY_PSEH ───────────────────────────────────────────────────────────────

extern "C" {
    /// `sdk/lib/pseh/dummy.c`'s always-zero global. msvcrt builds with DUMMY_PSEH, where
    /// `_SEH2_EXCEPT(filter)` expands to `if (_SEH2_Volatile0 || (0 && (filter)))` — a
    /// branch that is COMPILED but never taken, because nothing ever writes the variable.
    /// So `__TRY { A } __EXCEPT_PAGE_FAULT { B } __ENDTRY` compiles to `A;` followed by a
    /// runtime-false `if` around `B`, and that is exactly what the Rust must be.
    ///
    /// Reading it here rather than deleting the handler is deliberate, and cheap: the load
    /// is one instruction, the C emits the same one, and the alternative — dropping `B` as
    /// "unreachable" — would silently discard the only record of what cpp.c intends to do
    /// when an RTTI walk faults. It also keeps `B` compiling, so it cannot rot.
    ///
    /// rustconv's --seal-calls got this wrong in the other direction: its invariant is
    /// "every call in the C appears and is EXECUTABLE", and the model satisfied that by
    /// emitting the handler as straight-line code — which made `__RTtypeid`,
    /// `__RTCastToVoid` and `__RTDynamicCast` throw `__non_rtti_object` unconditionally,
    /// on every call, on every arch.
    pub static _SEH2_Volatile0: c_int;
}
