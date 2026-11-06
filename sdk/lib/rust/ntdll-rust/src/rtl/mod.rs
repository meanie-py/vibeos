//! rtl/ — main ntdll.dll rtl-glue translations.
//! Mirrors the C layout `dll/ntdll/rtl/` (the `rtl_um` OBJECT lib). These
//! land in ntdll.dll proper (NOT the ntdll_vista companion).

pub mod uilist;
pub mod version;
pub mod openkeyex;
#[cfg(not(feature = "vista"))]
pub mod procnum; // NtGetCurrentProcessorNumberEx (no kernel service; built over the plain one)
#[cfg(all(target_arch = "x86", not(feature = "vista")))]
pub mod slistex; // RtlInterlockedPushListSListEx: i386's stdcall face of the fastcall body
pub mod ktm;
#[cfg(not(feature = "vista"))]
pub mod fls; // RtlFls* (upstream fls.c, compiled into rtl): ntdll.dll only, spec-gated 0x600+/0xA00+
pub mod ntuserpfn; // Rtl*NtUserPfn (upstream rtl_vista): ntdll_vista.spec exports it, ntdll.spec at 0x600+
pub mod libsupp;
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
pub mod wow64shim; // WOW64 host surface, arm64+amd64 (docs/engineering/wow64-plan.md Phase A/E)
