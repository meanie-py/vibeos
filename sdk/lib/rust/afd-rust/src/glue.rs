//! Harvested afd #define consts / type aliases / macro shims the per-arch bindgen
//! substrate (`types`) doesn't emit. Values match drivers/network/afd/include/
//! {afd,debug}.h. The struct + most flag consts (AFD_FCB, FUNCTION_*, SOCKET_STATE_*,
//! AFD_EVENT_*, P-prefixed pointer aliases) ARE in `types`; only the items below are
//! genuinely absent (CCHAR/UINT base aliases, the multichar TAG_AFD_* pool tags that
//! bindgen skips, the debug trace levels, and the release-noop ASSERT/MIN/MAX macros
//! the C source uses as call expressions).

// --- base type aliases not in the kernel bindgen ---------------------------
pub type CCHAR = i8;
pub type UINT = u32;

// --- debug trace levels (debug.h) ------------------------------------------
pub const MIN_TRACE: u32 = 0x00000001;
pub const MID_TRACE: u32 = 0x00000002;
pub const MAX_TRACE: u32 = 0x00000003;

// --- NTSTATUS codes the modules reference but ntkrnl-sys doesn't export -----
pub const STATUS_FILE_CLOSED: i32 = 0xC0000128u32 as i32;

// --- pool tags (afd.h) — C multichar 'BdfA' = (B<<24)|(d<<16)|(f<<8)|A. bindgen
//     drops implementation-defined multichar #defines, so re-emit them here. ----
const fn fourcc(s: &[u8; 4]) -> u32 {
    ((s[0] as u32) << 24) | ((s[1] as u32) << 16) | ((s[2] as u32) << 8) | (s[3] as u32)
}
pub const TAG_AFD_DATA_BUFFER: u32 = fourcc(b"BdfA");
pub const TAG_AFD_TRANSPORT_ADDRESS: u32 = fourcc(b"tdfA");
pub const TAG_AFD_SOCKET_CONTEXT: u32 = fourcc(b"XdfA");
pub const TAG_AFD_CONNECT_DATA: u32 = fourcc(b"cdfA");
pub const TAG_AFD_DISCONNECT_DATA: u32 = fourcc(b"ddfA");
pub const TAG_AFD_CONNECT_OPTIONS: u32 = fourcc(b"ocfA");
pub const TAG_AFD_DISCONNECT_OPTIONS: u32 = fourcc(b"odfA");
pub const TAG_AFD_SUPER_CONNECT_BUFFER: u32 = fourcc(b"sdfA");
pub const TAG_AFD_ACCEPT_QUEUE: u32 = fourcc(b"qafA");
pub const TAG_AFD_POLL_HANDLE: u32 = fourcc(b"hpfA");
pub const TAG_AFD_FCB: u32 = fourcc(b"cffA");
pub const TAG_AFD_ACTIVE_POLL: u32 = fourcc(b"pafA");
pub const TAG_AFD_EA_INFO: u32 = fourcc(b"aefA");
pub const TAG_AFD_STORED_DATAGRAM: u32 = fourcc(b"gsfA");
pub const TAG_AFD_SNMP_ADDRESS_INFO: u32 = fourcc(b"asfA");
pub const TAG_AFD_TDI_CONNECTION_INFORMATION: u32 = fourcc(b"cTfA");
pub const TAG_AFD_WSA_BUFFER: u32 = fourcc(b"bWfA");

// --- release-noop / value macros used as call expressions in the C source ---
// ASSERT(x) is empty in release (NDEBUG); the C uses it as a statement-call.
#[inline(always)]
pub fn ASSERT<T>(_cond: T) {}
// MIN/MAX(a,b) — afd uses the min/max value macros.
#[inline(always)]
pub fn MIN<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}
#[inline(always)]
pub fn MAX<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}
