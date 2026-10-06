//! The BTCpu64 contract, Rust side. The C side is
//! `sdk/include/reactos/wowx64/btcpu64.h` — the plugin is C++ and includes
//! that; these are hand mirrors of it, pinned by the size/offset asserts at
//! the bottom so a one-sided edit fails the build instead of a boot. Change
//! both together and bump `WOWX64_CONTRACT_VERSION`.

pub const WOWX64_CONTRACT_VERSION: u32 = 2;

/// Register arguments of a guest system service (r10, rdx, r8, r9).
pub const WOWX64_REGISTER_ARGS: usize = 4;

pub const WOWX64_CONTEXT_AMD64: u32 = 0x0010_0000;
pub const WOWX64_CONTEXT_CONTROL: u32 = WOWX64_CONTEXT_AMD64 | 0x01;
pub const WOWX64_CONTEXT_INTEGER: u32 = WOWX64_CONTEXT_AMD64 | 0x02;
pub const WOWX64_CONTEXT_SEGMENTS: u32 = WOWX64_CONTEXT_AMD64 | 0x04;
pub const WOWX64_CONTEXT_FLOATING_POINT: u32 = WOWX64_CONTEXT_AMD64 | 0x08;
pub const WOWX64_CONTEXT_FULL: u32 =
    WOWX64_CONTEXT_CONTROL | WOWX64_CONTEXT_INTEGER | WOWX64_CONTEXT_FLOATING_POINT;
pub const WOWX64_CONTEXT_ALL: u32 = WOWX64_CONTEXT_FULL | WOWX64_CONTEXT_SEGMENTS;

/// Flat 64-bit user selectors, as on amd64 Windows.
pub const WOWX64_SEG_CS: u16 = 0x33;
pub const WOWX64_SEG_DS: u16 = 0x2B;
/// amd64 Windows' user FS selector. Unused in long mode, but real contexts carry it.
pub const WOWX64_SEG_FS: u16 = 0x53;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WowX64M128A {
    pub Low: u64,
    pub High: i64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WowX64XsaveFormat {
    pub ControlWord: u16,
    pub StatusWord: u16,
    pub TagWord: u8,
    pub Reserved1: u8,
    pub ErrorOpcode: u16,
    pub ErrorOffset: u32,
    pub ErrorSelector: u16,
    pub Reserved2: u16,
    pub DataOffset: u32,
    pub DataSelector: u16,
    pub Reserved3: u16,
    pub MxCsr: u32,
    pub MxCsr_Mask: u32,
    pub FloatRegisters: [WowX64M128A; 8],
    pub XmmRegisters: [WowX64M128A; 16],
    pub Reserved4: [u8; 96],
}

/// The x86-64 `CONTEXT` (0x4D0 bytes). Named apart from `CONTEXT` because on
/// this host `CONTEXT` is the ARM64 one.
#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct WowX64Context {
    pub P1Home: u64,
    pub P2Home: u64,
    pub P3Home: u64,
    pub P4Home: u64,
    pub P5Home: u64,
    pub P6Home: u64,
    pub ContextFlags: u32,
    pub MxCsr: u32,
    pub SegCs: u16,
    pub SegDs: u16,
    pub SegEs: u16,
    pub SegFs: u16,
    pub SegGs: u16,
    pub SegSs: u16,
    pub EFlags: u32,
    pub Dr0: u64,
    pub Dr1: u64,
    pub Dr2: u64,
    pub Dr3: u64,
    pub Dr6: u64,
    pub Dr7: u64,
    pub Rax: u64,
    pub Rcx: u64,
    pub Rdx: u64,
    pub Rbx: u64,
    pub Rsp: u64,
    pub Rbp: u64,
    pub Rsi: u64,
    pub Rdi: u64,
    pub R8: u64,
    pub R9: u64,
    pub R10: u64,
    pub R11: u64,
    pub R12: u64,
    pub R13: u64,
    pub R14: u64,
    pub R15: u64,
    pub Rip: u64,
    pub FltSave: WowX64XsaveFormat,
    pub VectorRegister: [WowX64M128A; 26],
    pub VectorControl: u64,
    pub DebugControl: u64,
    pub LastBranchToRip: u64,
    pub LastBranchFromRip: u64,
    pub LastExceptionToRip: u64,
    pub LastExceptionFromRip: u64,
}

/// OS-side services handed to the CPU layer in `BTCpu64ProcessInit`.
#[repr(C)]
pub struct WowX64HostServices {
    pub Size: u32,
    pub Version: u32,
    /// Run guest service `ServiceNumber`: its register arguments in guest
    /// order, and the guest rsp at the syscall, from which the OS side reads
    /// the stack arguments the service actually takes. The result goes to guest
    /// RAX unchanged.
    pub SystemService:
        unsafe extern "C" fn(ServiceNumber: u32, RegisterArgs: *const u64, GuestRsp: u64) -> u64,
}

const _: () = assert!(core::mem::size_of::<WowX64XsaveFormat>() == 0x200);
const _: () = assert!(core::mem::size_of::<WowX64Context>() == 0x4D0);
const _: () = assert!(core::mem::offset_of!(WowX64Context, Rax) == 0x78);
const _: () = assert!(core::mem::offset_of!(WowX64Context, Rip) == 0xF8);
const _: () = assert!(core::mem::offset_of!(WowX64Context, FltSave) == 0x100);
const _: () = assert!(core::mem::offset_of!(WowX64Context, VectorRegister) == 0x300);
