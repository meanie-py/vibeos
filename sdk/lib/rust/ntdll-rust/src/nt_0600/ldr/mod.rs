//! nt_0600/ldr/ — ntdll_vista.dll loader-support translations.
//! Mirrors the C layout `dll/ntdll/nt_0600/ldr/`. The C lived in
//! `ntdll_vista_static`, which is statically linked into BOTH ntdll.dll and
//! ntdll_vista.dll — so these modules are UNGATED (compiled in both builds).

pub mod ldrinit;
// DLL load/unload notification registry. UNGATED: LdrpSendDllNotifications is
// called by the main ntdll.dll loader; LdrRegister/UnregisterDllNotification are
// exported by ntdll_vista.dll. Mirrors ldrinit (LdrpHeap) — both builds need it.
pub mod ldrnotify;
