//! Timezone constants for `cabi::Rtl{Query,Set}TimeZoneInformation`.
//!
//! `timezone.c` is pure registry plumbing — no algorithmic content
//! to host in `core_rs`. This module just centralises the registry
//! key path + value names + flag/type constants so the `cabi` shims
//! stay tidy.
//!
//! Faithful to `sdk/lib/rtl/timezone.c` (ReactOS, Eric Kohl).

/// `RTL_REGISTRY_CONTROL` — `\Registry\Machine\System\CurrentControlSet\Control`.
pub const RTL_REGISTRY_CONTROL: u32 = 2;

/// `RTL_QUERY_REGISTRY_DIRECT` — write straight into `EntryContext`.
pub const RTL_QUERY_REGISTRY_DIRECT: u32 = 0x0000_0020;

/// `REG_SZ` value type.
pub const REG_SZ: u32 = 1;
/// `REG_BINARY` value type.
pub const REG_BINARY: u32 = 3;
/// `REG_DWORD` value type.
pub const REG_DWORD: u32 = 4;

/// Number of `WCHAR` slots in `RTL_TIME_ZONE_INFORMATION.StandardName` /
/// `DaylightName` (both `WCHAR[32]`).
pub const TZ_NAME_CHARS: u16 = 32;

/// `sizeof(SYSTEMTIME)` — 8 × `WORD` = 16 bytes. The C code passes
/// `sizeof(SYSTEMTIME)` to `RtlWriteRegistryValue` for the
/// `StandardDate` / `DaylightDate` fields (both `TIME_FIELDS`, which
/// has the same 16-byte layout).
pub const SIZEOF_SYSTEMTIME: u32 = 16;
