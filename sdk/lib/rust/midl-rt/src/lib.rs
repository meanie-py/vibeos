//! Rust type substrate for widl/MIDL-generated RPC **server** stubs.
//!
//! A widl-generated `<iface>_s.c` instantiates a graph of `repr(C)` statics
//! drawn from `rpcndr.h` / `rpcdcep.h` — `MIDL_STUB_DESC`, `MIDL_SERVER_INFO`,
//! `RPC_SERVER_INTERFACE`, `RPC_DISPATCH_TABLE`, the format-string wrappers —
//! plus a couple of tiny `ExprEval` routines that poke `MIDL_STUB_MESSAGE`.
//! This crate is the faithful Rust mirror of exactly those types, so the
//! `widl_to_rust.py`-generated `<iface>_s.rs` can build the same static graph
//! with no C. The NDR engine (rpcrt4) walks this graph at dispatch time, so
//! every field offset must match the C ABI bit-for-bit — the `const _` size and
//! offset assertions below are the guard (an offset mismatch is a silent RPC
//! corruption / hang, caught nowhere else).
//!
//! Sizes/offsets were extracted from the in-tree `rpcndr.h`/`rpcdcep.h` via
//! clang for all three targets:
//!   struct              i386   amd64 / arm64
//!   MIDL_STUB_DESC       80     152
//!   MIDL_SERVER_INFO     32      64
//!   RPC_SERVER_INTERFACE 68      96
//!   RPC_DISPATCH_TABLE   12      24
//!   RPC_SYNTAX_IDENTIFIER 20     20
//!   STUB_MESSAGE.MaxCount@ 60   104  (ULONG_PTR)
//!   STUB_MESSAGE.Offset@   64   112  (ULONG)
//!   STUB_MESSAGE.StackTop@ 80   136  (uchar*)

#![no_std]
#![allow(non_camel_case_types)]

use core::ffi::c_void;

/// `RPC_MESSAGE` — opaque to the stub (only ever passed by pointer to
/// `NdrServerCall2`). Size/layout irrelevant on this side.
#[repr(C)]
pub struct RPC_MESSAGE {
    _opaque: [u8; 0],
}

/// Function-pointer typedefs. All MIDL stub callbacks are `__RPC_USER` /
/// `__RPC_API` / `__RPC_STUB`, i.e. `__stdcall` on i386 and cdecl elsewhere —
/// `extern "system"` resolves to exactly that per arch.
pub type RPC_DISPATCH_FUNCTION = unsafe extern "system" fn(*mut RPC_MESSAGE);
pub type SERVER_ROUTINE = unsafe extern "system" fn() -> i32;
pub type NDR_RUNDOWN = unsafe extern "system" fn(*mut c_void);
pub type EXPR_EVAL = unsafe extern "system" fn(*mut MIDL_STUB_MESSAGE);

/// `GUID` — 16 bytes, align 4.
#[repr(C)]
pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

/// `RPC_VERSION` — `{ unsigned short MajorVersion; unsigned short MinorVersion; }`.
#[repr(C)]
pub struct RPC_VERSION {
    pub major: u16,
    pub minor: u16,
}

/// `RPC_SYNTAX_IDENTIFIER` — `{ GUID SyntaxGUID; RPC_VERSION SyntaxVersion; }`.
#[repr(C)]
pub struct RPC_SYNTAX_IDENTIFIER {
    pub syntax_guid: GUID,
    pub syntax_version: RPC_VERSION,
}

/// `RPC_DISPATCH_TABLE` (rpcdcep.h).
#[repr(C)]
pub struct RPC_DISPATCH_TABLE {
    pub dispatch_table_count: u32,
    /// `RPC_DISPATCH_FUNCTION *` — points at a `[*const c_void; N]` table.
    pub dispatch_table: *const c_void,
    pub reserved: isize,
}
unsafe impl Sync for RPC_DISPATCH_TABLE {}

/// `RPC_SERVER_INTERFACE` (rpcdcep.h).
#[repr(C)]
pub struct RPC_SERVER_INTERFACE {
    pub length: u32,
    pub interface_id: RPC_SYNTAX_IDENTIFIER,
    pub transfer_syntax: RPC_SYNTAX_IDENTIFIER,
    /// `PRPC_DISPATCH_TABLE`
    pub dispatch_table: *const c_void,
    pub rpc_protseq_endpoint_count: u32,
    /// `PRPC_PROTSEQ_ENDPOINT`
    pub rpc_protseq_endpoint: *const c_void,
    /// `RPC_MGR_EPV *`
    pub default_manager_epv: *const c_void,
    /// `void const *` (the `MIDL_SERVER_INFO`)
    pub interpreter_info: *const c_void,
    pub flags: u32,
}
unsafe impl Sync for RPC_SERVER_INTERFACE {}

/// `MIDL_STUB_DESC` (rpcndr.h). Every pointer field is rendered `*const c_void`
/// (the stub only ever stores addresses of other statics / extern routines).
/// The three scalar fields keep their exact C types.
#[repr(C)]
pub struct MIDL_STUB_DESC {
    pub rpc_interface_information: *const c_void,
    pub pfn_allocate: *const c_void,
    pub pfn_free: *const c_void,
    /// `IMPLICIT_HANDLE_INFO` union — one pointer.
    pub implicit_handle_info: *const c_void,
    pub apfn_ndr_rundown_routines: *const c_void,
    pub a_generic_binding_routine_pairs: *const c_void,
    pub apfn_expr_eval: *const c_void,
    pub a_xmit_quintuple: *const c_void,
    pub p_format_types: *const c_void,
    pub f_check_bounds: i32,
    pub version: u32,
    pub p_malloc_free_struct: *const c_void,
    pub midl_version: i32,
    pub comm_fault_offsets: *const c_void,
    pub a_user_marshal_quadruple: *const c_void,
    pub notify_routine_table: *const c_void,
    pub m_flags: usize,
    pub cs_routine_tables: *const c_void,
    pub proxy_server_info: *const c_void,
    pub p_expr_info: *const c_void,
}
unsafe impl Sync for MIDL_STUB_DESC {}

/// `MIDL_SERVER_INFO` (rpcndr.h).
#[repr(C)]
pub struct MIDL_SERVER_INFO {
    pub p_stub_desc: *const c_void,
    /// `const SERVER_ROUTINE *` — points at a `[*const c_void; N]` table.
    pub dispatch_table: *const c_void,
    /// `PFORMAT_STRING` (proc format string `.Format`)
    pub proc_string: *const c_void,
    /// `const unsigned short *` (FormatStringOffsetTable)
    pub fmt_string_offset: *const c_void,
    pub thunk_table: *const c_void,
    pub p_transfer_syntax: *const c_void,
    pub n_count: usize,
    pub p_syntax_info: *const c_void,
}
unsafe impl Sync for MIDL_SERVER_INFO {}

/// `MIDL_STUB_MESSAGE` — modeled faithfully only through `StackTop` (the last
/// field any `ExprEval` touches); the tail is irrelevant on this side. The
/// offset assertions below lock `MaxCount`/`Offset`/`StackTop` to the C ABI.
#[repr(C)]
pub struct MIDL_STUB_MESSAGE {
    pub rpc_msg: *mut c_void,
    pub buffer: *mut u8,
    pub buffer_start: *mut u8,
    pub buffer_end: *mut u8,
    pub buffer_mark: *mut u8,
    pub buffer_length: u32,
    pub memory_size: u32,
    pub memory: *mut u8,
    pub is_client: u8,
    pub pad: u8,
    pub u_flags2: u16,
    pub reuse_buffer: i32,
    pub p_alloc_all_nodes_context: *mut c_void,
    pub p_pointer_queue_state: *mut c_void,
    pub ignore_embedded_pointers: i32,
    pub pointer_buffer_mark: *mut u8,
    pub corr_desp_increment: u8,
    pub u_flags: u8,
    pub unique_ptr_count: u16,
    pub max_count: usize,
    pub offset: u32,
    pub actual_count: u32,
    pub pfn_allocate: *mut c_void,
    pub pfn_free: *mut c_void,
    pub stack_top: *mut u8,
}

/// `MIDL_PROC_FORMAT_STRING` / `MIDL_TYPE_FORMAT_STRING` — `{ short Pad;
/// unsigned char Format[]; }`. Parameterized over the byte-array length the
/// stub emits. No pointers ⇒ auto-`Sync`.
#[repr(C)]
pub struct MidlFormatString<const N: usize> {
    pub pad: i16,
    pub format: [u8; N],
}

/// `Sync` newtype over a function-address / pointer table. Every MIDL function
/// table — server routines (`(void*)Routine`), dispatch functions
/// (`NdrServerCall2` …, trailing null), `EXPR_EVAL[]`, `NDR_RUNDOWN[]` — is an
/// array of code addresses; rendered uniformly as `[*const c_void; N]`. A bare
/// raw-pointer array isn't `Sync`, hence the wrapper.
#[repr(transparent)]
pub struct PtrTable<const N: usize>(pub [*const c_void; N]);
unsafe impl<const N: usize> Sync for PtrTable<N> {}

/// `Sync` newtype for the exported `RPC_IF_HANDLE <iface>_s_ifspec` data symbol
/// (its stored value is `&<iface>___RpcServerInterface`).
#[repr(transparent)]
pub struct RpcIfHandle(pub *const c_void);
unsafe impl Sync for RpcIfHandle {}

/// `Sync` newtype over the protseq-endpoint table
/// (`unsigned char const *[][2]` — `{ protseq, endpoint }` C-string pairs).
#[repr(C)]
pub struct ProtseqEndpoints<const N: usize>(pub [[*const u8; 2]; N]);
unsafe impl<const N: usize> Sync for ProtseqEndpoints<N> {}

// ---- ABI guards: sizes + the three ExprEval-touched offsets ----------------
const _: () = {
    assert!(core::mem::size_of::<RPC_SYNTAX_IDENTIFIER>() == 20);
};

#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<MIDL_STUB_DESC>() == 80);
    assert!(core::mem::size_of::<MIDL_SERVER_INFO>() == 32);
    assert!(core::mem::size_of::<RPC_SERVER_INTERFACE>() == 68);
    assert!(core::mem::size_of::<RPC_DISPATCH_TABLE>() == 12);
    assert!(core::mem::offset_of!(MIDL_STUB_MESSAGE, max_count) == 60);
    assert!(core::mem::offset_of!(MIDL_STUB_MESSAGE, offset) == 64);
    assert!(core::mem::offset_of!(MIDL_STUB_MESSAGE, stack_top) == 80);
};

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<MIDL_STUB_DESC>() == 152);
    assert!(core::mem::size_of::<MIDL_SERVER_INFO>() == 64);
    assert!(core::mem::size_of::<RPC_SERVER_INTERFACE>() == 96);
    assert!(core::mem::size_of::<RPC_DISPATCH_TABLE>() == 24);
    assert!(core::mem::offset_of!(MIDL_STUB_MESSAGE, max_count) == 104);
    assert!(core::mem::offset_of!(MIDL_STUB_MESSAGE, offset) == 112);
    assert!(core::mem::offset_of!(MIDL_STUB_MESSAGE, stack_top) == 136);
};
