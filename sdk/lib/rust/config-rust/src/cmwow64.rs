//! WoW64 registry redirection — the Wow6432Node view for 32-bit processes.
//!
//! ONE RULE, ONE PLACE: when a 32-bit process's key open crosses
//! `\REGISTRY\MACHINE\SOFTWARE` into a child, `WOW6432NODE` is spliced in, so
//! the process reads and writes `\REGISTRY\MACHINE\SOFTWARE\WOW6432NODE\...`.
//! Every key open in the system funnels through `CmpParseKey` — Ob's only
//! parse-procedure bypass is for directory objects, and a key is never one —
//! so that wrapper is a complete interception point. Nothing above CM needs to
//! know about redirection, and a caller that reaches `NtOpenKey` directly is
//! covered exactly like one that came through advapi32.
//!
//! MISS FALLS BACK TO THE 64-BIT KEY. A redirected open that comes back
//! OBJECT_NAME_NOT_FOUND is retried unredirected (non-creates only — a create
//! must land in the 32-bit view or it isn't isolation). Note what that means
//! precisely: isolation comes from CREATE, not from access mask. A 32-bit
//! process that OPENS an existing 64-bit-only key for write falls back and
//! writes the shared key, where Windows would have failed the open. That is the
//! direct cost of not populating, and it is the safe direction — the
//! alternative breaks every 32-bit app that updates an OS key.
//!
//! This is the Wine model, and it is what makes the feature safe to enable on a
//! system that ships ONE set of components: writes get a private node, reads
//! still find the OS keys that only exist 64-bit. Real Windows hides them and
//! populates the 32-bit
//! view from the installer — which we cannot do, because nothing populates
//! Wow6432Node here. Redirection without population is a regression, and this
//! is the cheap half of the trade: no 1199-line hive mirror to drift out of
//! date, at the cost of not reproducing Windows' "32-bit app genuinely cannot
//! see this key" behaviour. Revisit if we ever ship 32-bit-only components.
//!
//! The exemption list is deliberately tiny (see `CmpWow64IsSharedKey`), because
//! the fallback already covers what an exemption would: a shared key that has
//! no 32-bit copy resolves to the 64-bit one anyway.

use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{
    ACCESS_MASK, CmpConstructName, CmpFree, PACCESS_STATE, PCM_KEY_BODY,
    PCM_KEY_CONTROL_BLOCK, PUNICODE_STRING, _UNICODE_STRING,
};
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::PagedPool;
use ntkrnl_sys::opaque_ptr;

use crate::cmsysini::{CmpLockRegistry, CmpUnlockRegistry};
use crate::inlines::{CmpAcquireKcbLockShared, CmpReleaseKcbLock};

/// KEY_WOW64_* (winnt.h). These are VIEW SELECTORS, not access bits: no key's
/// ACL grants either, so one left set denies the open outright. They must be
/// consumed and cleared before `ObCheckObjectAccess` runs. That strip used to
/// live in NtOpenKey/NtCreateKey (E21/#79, where NtOpenKey masked a bare 0x0100
/// and let KEY_WOW64_32KEY through to the access check). It moved here so the
/// flags survive long enough to pick a view — same invariant, enforced at the
/// single choke point every key open passes through instead of at two callers.
const KEY_WOW64_64KEY: ACCESS_MASK = 0x00000100;
const KEY_WOW64_32KEY: ACCESS_MASK = 0x00000200;
const KEY_WOW64_RES: ACCESS_MASK = 0x00000300;

// L"\\REGISTRY\\MACHINE\\SOFTWARE" — 26 WCHARs + NUL.
static REDIRECT_ROOT: [u16; 27] = [
    b'\\' as u16, b'R' as u16, b'E' as u16, b'G' as u16, b'I' as u16, b'S' as u16,
    b'T' as u16, b'R' as u16, b'Y' as u16, b'\\' as u16, b'M' as u16, b'A' as u16,
    b'C' as u16, b'H' as u16, b'I' as u16, b'N' as u16, b'E' as u16, b'\\' as u16,
    b'S' as u16, b'O' as u16, b'F' as u16, b'T' as u16, b'W' as u16, b'A' as u16,
    b'R' as u16, b'E' as u16, 0
];

// L"\\WOW6432NODE" — 12 WCHARs + NUL.
static VIEW_NODE: [u16; 13] = [
    b'\\' as u16, b'W' as u16, b'O' as u16, b'W' as u16, b'6' as u16, b'4' as u16,
    b'3' as u16, b'2' as u16, b'N' as u16, b'O' as u16, b'D' as u16, b'E' as u16,
    0
];

/// Pool tag 'Rw64' — the spliced remaining-name buffer.
const TAG_WOW64_REDIRECT: u32 = u32::from_le_bytes(*b"46wR");

/// Length of `REDIRECT_ROOT` in WCHARs, NUL excluded.
const REDIRECT_ROOT_LEN: usize = REDIRECT_ROOT.len() - 1;
/// Length of `VIEW_NODE` in WCHARs, NUL excluded (leading separator included).
const VIEW_NODE_LEN: usize = VIEW_NODE.len() - 1;

/// ASCII-case-insensitive compare of two WCHAR runs. Registry path components
/// on this path are ASCII (`\REGISTRY\MACHINE\SOFTWARE`, `WOW6432NODE`), so a
/// full RtlUpcaseUnicodeString is not needed and would cost an allocation.
#[inline]
unsafe fn eq_ci(a: *const u16, b: *const u16, len: usize) -> bool {
    for i in 0..len {
        let mut ca = *a.add(i);
        let mut cb = *b.add(i);
        if (b'a' as u16..=b'z' as u16).contains(&ca) {
            ca -= 0x20;
        }
        if (b'a' as u16..=b'z' as u16).contains(&cb) {
            cb -= 0x20;
        }
        if ca != cb {
            return false;
        }
    }
    true
}

/// Is `Name[at..]` the start of a `WOW6432NODE` component? Used for
/// idempotence: a caller that spelled the node itself must not get a second
/// one spliced in, or `SOFTWARE\WOW6432NODE\WOW6432NODE` results.
#[inline]
unsafe fn is_view_node_at(Name: *const u16, NameLen: usize, at: usize) -> bool {
    // VIEW_NODE carries a leading separator; compare against the bare name.
    let node = VIEW_NODE.as_ptr().add(1);
    let node_len = VIEW_NODE_LEN - 1;
    if NameLen < at + node_len {
        return false;
    }
    if !eq_ci(Name.add(at), node, node_len) {
        return false;
    }
    // Must be a whole component, not a prefix of e.g. "Wow6432NodeX".
    NameLen == at + node_len || *Name.add(at + node_len) == b'\\' as u16
}

/// Does the parse want the 32-bit view? Reads the WoW64 selector bits, then
/// clears them from the access state so they can never reach the access check.
///
/// Native process + KEY_WOW64_32KEY selects the 32-bit view (that is how a
/// 64-bit tool inspects it — regedit's "32-bit view", ole32's HKCR handling).
/// A WoW64 process gets the 32-bit view by default, and KEY_WOW64_64KEY is its
/// only way back out to the 64-bit one — so honouring that flag is not a
/// refinement, it is what keeps the 64-bit view reachable at all.
pub(crate) unsafe fn CmpWow64ConsumeViewFlags(AccessState: PACCESS_STATE) -> bool {
    let mut Requested: ACCESS_MASK = 0;

    if !AccessState.is_null() {
        Requested = (*AccessState).RemainingDesiredAccess | (*AccessState).OriginalDesiredAccess;
        (*AccessState).RemainingDesiredAccess &= !KEY_WOW64_RES;
        (*AccessState).OriginalDesiredAccess &= !KEY_WOW64_RES;
    }

    // A 32-bit kernel has one registry view and no WoW64 processes, so both
    // flags are no-ops there — exactly as on 32-bit Windows. Clearing them
    // above is still required: E21's ACCESS_DENIED is not arch-specific.
    if cfg!(target_arch = "x86") {
        return false;
    }

    if (Requested & KEY_WOW64_64KEY) != 0 {
        return false;
    }
    if (Requested & KEY_WOW64_32KEY) != 0 {
        return true;
    }

    CmpWow64IsCurrentProcess32Bit()
}

/// Is the calling process a 32-bit (WoW64) process?
///
/// EPROCESS.Wow64Process is the PEB32 pointer the kernel hands a WoW64 process;
/// it is NULL for every native process, so this is also the gate that keeps the
/// whole feature off the native boot path — smss, csrss, winlogon and explorer
/// are all 64-bit and never see a redirect.
///
/// Split by arch rather than cfg'd at the call site: a 32-bit kernel's EPROCESS
/// has no Wow64Process member at all, so the field reference itself has to be
/// compiled out, not just the branch that uses it.
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn CmpWow64IsCurrentProcess32Bit() -> bool {
    extern "system" {
        #[link_name = "IoGetCurrentProcess"]
        fn IoGetCurrentProcess() -> *mut ntkrnl_sys::ddk::_EPROCESS;
    }

    let Process = IoGetCurrentProcess();
    !Process.is_null() && !(*Process).Wow64Process.is_null()
}

#[cfg(target_arch = "x86")]
#[inline]
unsafe fn CmpWow64IsCurrentProcess32Bit() -> bool {
    false
}

/// Subkeys of `\REGISTRY\MACHINE\SOFTWARE` that are NOT redirected.
///
/// Only the view node itself, for now. Windows keeps a long shared list here
/// (Policies, Registered Applications, parts of Classes...), but every entry on
/// it exists to stop a 32-bit app being cut off from a key it needs — which the
/// miss-fallback in `CmpParseKey` already does, for every key, without a table
/// to keep in sync. What an exemption adds on top is that 32-bit *writes* also
/// land in the shared key; add a name here when we find one that needs that.
#[inline]
unsafe fn CmpWow64IsSharedKey(Name: *const u16, NameLen: usize) -> bool {
    is_view_node_at(Name, NameLen, 0)
}


/// A spliced remaining-name. Owns its pool buffer; `free()` is mandatory.
pub(crate) struct CmpWow64Name {
    pub Name: _UNICODE_STRING,
    Buffer: PWSTR,
}

impl CmpWow64Name {
    pub(crate) unsafe fn free(&mut self) {
        if !self.Buffer.is_null() {
            ExFreePoolWithTag(self.Buffer as PVOID, TAG_WOW64_REDIRECT);
            self.Buffer = core::ptr::null_mut();
        }
    }
}

/// Build the redirected form of `RemainingName`, or return None if this open
/// does not cross the redirection boundary.
///
/// The boundary is crossed only when the `SOFTWARE` component lands INSIDE
/// `RemainingName` — i.e. the caller's root handle is a proper ancestor of
/// `\REGISTRY\MACHINE\SOFTWARE`. If the handle is already at or below SOFTWARE
/// there is nothing to decide: that handle was itself resolved under some view
/// and relative opens stay in it. This is what gives handles a sticky view for
/// free, and it matches Windows — a 32-bit app that opened SOFTWARE with
/// KEY_WOW64_64KEY keeps the 64-bit view for everything it opens beneath.
pub(crate) unsafe fn CmpWow64BuildRedirectedName(
    ParseObject: PVOID,
    RemainingName: PUNICODE_STRING,
) -> Option<CmpWow64Name> {
    let ParseObject = opaque_ptr(ParseObject);
    let RemainingName = opaque_ptr(RemainingName);

    if ParseObject.is_null() || RemainingName.is_null() {
        return None;
    }

    let RemBuf = (*RemainingName).Buffer;
    let RemLen = (*RemainingName).Length as usize / core::mem::size_of::<u16>();
    if RemBuf.is_null() || RemLen == 0 {
        return None;
    }

    let Kcb: PCM_KEY_CONTROL_BLOCK = (*(ParseObject as PCM_KEY_BODY)).KeyControlBlock;
    if Kcb.is_null() {
        return None;
    }


    // CmpConstructName walks the KCB parent chain and reads hive cells, so it
    // needs the registry lock AND the KCB shared, with a delete check between —
    // exactly the sequence CmpQueryKeyName uses (cmsysini.c). We are ahead of
    // CmpParseKey's own locking here (CmpBuildHashStackAndLookupCache takes the
    // registry lock), so this acquires and drops its own.
    CmpLockRegistry();
    CmpAcquireKcbLockShared(Kcb);
    let ParentName = if (*Kcb).__bindgen_anon_1.Delete() != 0 {
        core::ptr::null_mut()
    } else {
        CmpConstructName(Kcb)
    };
    CmpReleaseKcbLock(Kcb);
    CmpUnlockRegistry();

    if ParentName.is_null() {
        return None;
    }
    let ParentLen = (*ParentName).Length as usize / core::mem::size_of::<u16>();
    let ParentBuf = (*ParentName).Buffer;

    // The parent must be a PROPER ancestor of the redirect root: its path is a
    // prefix of the root, and the root continues with a separator.
    let matched = ParentLen < REDIRECT_ROOT_LEN
        && !ParentBuf.is_null()
        && eq_ci(ParentBuf, REDIRECT_ROOT.as_ptr(), ParentLen)
        && REDIRECT_ROOT[ParentLen] == b'\\' as u16;

    CmpFree(ParentName as PVOID, 0);

    if !matched {
        return None;
    }

    // Ob hands the remaining name over in two different shapes, and both occur
    // on every boot: WITH a leading separator when the open was by absolute
    // path (parent `\REGISTRY`, remaining `\Machine\Software\...`), and WITHOUT
    // one when it was relative to a key handle (parent `\REGISTRY\MACHINE`,
    // remaining `SYSTEM\...`). Assuming the second shape is what made the first
    // cut of this feature inert: every absolute open compared `\Machine\Softwar`
    // against `MACHINE\SOFTWARE` and silently declined to redirect.
    let mut Lead: usize = 0;
    while Lead < RemLen && *RemBuf.add(Lead) == b'\\' as u16 {
        Lead += 1;
    }

    // How much of the root the remaining name still has to supply. The +1 skips
    // the separator that joins parent and remaining name.
    let Want = REDIRECT_ROOT_LEN - (ParentLen + 1);
    let split = Lead + Want;
    if split > RemLen {
        return None;
    }

    // RemainingName must actually spell the rest of the root...
    if !eq_ci(RemBuf.add(Lead), REDIRECT_ROOT.as_ptr().add(ParentLen + 1), Want) {
        return None;
    }
    // ...and stop on a component boundary.
    if split != RemLen && *RemBuf.add(split) != b'\\' as u16 {
        return None;
    }

    // Already inside a view, or naming a shared key: leave it alone.
    if split != RemLen && CmpWow64IsSharedKey(RemBuf.add(split + 1), RemLen - split - 1) {
        return None;
    }

    // Splice: <root remainder> \WOW6432NODE <rest>
    let NewLen = RemLen + VIEW_NODE_LEN;
    if NewLen * core::mem::size_of::<u16>() > u16::MAX as usize {
        return None;
    }
    let Bytes = NewLen * core::mem::size_of::<u16>();
    let NewBuf = ExAllocatePoolWithTag(PagedPool, Bytes, TAG_WOW64_REDIRECT) as PWSTR;
    if NewBuf.is_null() {
        return None;
    }

    core::ptr::copy_nonoverlapping(RemBuf, NewBuf, split);
    core::ptr::copy_nonoverlapping(VIEW_NODE.as_ptr(), NewBuf.add(split), VIEW_NODE_LEN);
    core::ptr::copy_nonoverlapping(
        RemBuf.add(split),
        NewBuf.add(split + VIEW_NODE_LEN),
        RemLen - split,
    );

    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    Name.Buffer = NewBuf;
    Name.Length = Bytes as u16;
    Name.MaximumLength = Bytes as u16;

    Some(CmpWow64Name { Name, Buffer: NewBuf })
}
