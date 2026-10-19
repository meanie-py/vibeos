//! A 32-bit winspool under WoW64 talks to the NATIVE spooler, and the spooler flattens every
//! info structure it returns (`GetPrinter`, `EnumJobs`, `GetPrinterDriver`, ...) into the RPC
//! buffer in ITS layout: 8-byte pointer offsets, 64-bit padding. `MarshallUpStructure` then read
//! that with the i386 table, so every field after the first pointer came from the wrong bytes.
//! wordpad died of it: `PRINTDLG_SetDefaultPrinter`'s `pbuf->pDevMode` read the zero high half
//! of `pPortName`'s offset, and `pDevMode->dmSize` faulted at 0x44.
//!
//! Windows' spooler narrows for a 32-bit caller itself. Here the client does it, because the
//! client is the side that knows: `NarrowSpoolerStructures` rewrites the buffer in place into
//! the i386 layout, rebasing each element's offsets for its new position, before the ordinary
//! unmarshalling runs. It needs the 64-bit layout of every structure, and the marshalling
//! tables cannot supply it: they omit fields (`DRIVER_INFO_x.cVersion`, `ftDriverDate`), and
//! i386 pads `dwlDriverVersion` where amd64 does not. So each structure has a twin below, its
//! fields in order with every pointer an 8-byte slot. On amd64 and arm64 the twin is checked
//! against the real structure field by field at compile time, so a wrong or missing field does
//! not build; i386 aligns `u64` to 8 as well, so the same twin gives the spooler's layout there.

#![cfg_attr(not(target_arch = "x86"), allow(dead_code))]

use crate::prelude::*;
use core::mem::{offset_of, size_of};

#[derive(Clone, Copy)]
pub(crate) struct TwinField {
    /// Offset in this build's (i386) structure.
    at: u32,
    /// Offset in the spooler's (64-bit) structure.
    from: u32,
    /// Bytes to copy for a value field.
    size: u32,
    pointer: bool,
}

pub(crate) struct Twin {
    size32: u32,
    size64: u32,
    fields: &'static [TwinField],
}

/// The size of the field `f` selects, without a value to take it from.
const fn field_size<T, F>(_f: fn(&T) -> &F) -> usize {
    size_of::<F>()
}

macro_rules! twins {
    ($( $twin:ident = $real:ident as $layout:ident { $($kind:ident $f:ident $(: $t:ty)?),* $(,)? } )*) => { $(
        #[repr(C)]
        #[allow(non_snake_case, non_camel_case_types, dead_code)]
        struct $layout { $($f: twins!(@ty $kind $($t)?)),* }

        #[cfg(not(target_arch = "x86"))]
        const _: () = {
            $(assert!(offset_of!($layout, $f) == offset_of!($real, $f));)*
            $(assert!(field_size(|s: &$layout| &s.$f) == field_size(|s: &$real| &s.$f));)*
            assert!(size_of::<$layout>() == size_of::<$real>());
        };
        // A `ptr` is a pointer here too, not some other 8-byte field.
        #[cfg(target_arch = "x86")]
        const _: () = {
            $(assert!(!twins!(@ptr $kind) || field_size(|s: &$real| &s.$f) == 4);)*
            $(assert!(twins!(@ptr $kind) || field_size(|s: &$real| &s.$f) == field_size(|s: &$layout| &s.$f));)*
        };

        static $twin: Twin = Twin {
            size32: size_of::<$real>() as u32,
            size64: size_of::<$layout>() as u32,
            fields: &[$(TwinField {
                at: offset_of!($real, $f) as u32,
                from: offset_of!($layout, $f) as u32,
                size: size_of::<twins!(@ty $kind $($t)?)>() as u32,
                pointer: twins!(@ptr $kind),
            }),*],
        };
    )* };
    (@ty ptr) => { u64 };
    (@ty val $t:ty) => { $t };
    (@ptr ptr) => { true };
    (@ptr val) => { false };
}

#[cfg(target_arch = "x86")]
const _: () = assert!(core::mem::align_of::<u64>() == 8);

twins! {
    PRINTER_INFO_1W_TWIN = PRINTER_INFO_1W as PRINTER_INFO_1W_64 {
        val Flags: DWORD, ptr pDescription, ptr pName, ptr pComment,
    }
    PRINTER_INFO_2W_TWIN = PRINTER_INFO_2W as PRINTER_INFO_2W_64 {
        ptr pServerName, ptr pPrinterName, ptr pShareName, ptr pPortName, ptr pDriverName,
        ptr pComment, ptr pLocation, ptr pDevMode, ptr pSepFile, ptr pPrintProcessor,
        ptr pDatatype, ptr pParameters, ptr pSecurityDescriptor, val Attributes: DWORD,
        val Priority: DWORD, val DefaultPriority: DWORD, val StartTime: DWORD,
        val UntilTime: DWORD, val Status: DWORD, val cJobs: DWORD, val AveragePPM: DWORD,
    }
    PRINTER_INFO_3_TWIN = PRINTER_INFO_3 as PRINTER_INFO_3_64 { ptr pSecurityDescriptor }
    PRINTER_INFO_4W_TWIN = PRINTER_INFO_4W as PRINTER_INFO_4W_64 {
        ptr pPrinterName, ptr pServerName, val Attributes: DWORD,
    }
    PRINTER_INFO_5W_TWIN = PRINTER_INFO_5W as PRINTER_INFO_5W_64 {
        ptr pPrinterName, ptr pPortName, val Attributes: DWORD,
        val DeviceNotSelectedTimeout: DWORD, val TransmissionRetryTimeout: DWORD,
    }
    PRINTER_INFO_6_TWIN = PRINTER_INFO_6 as PRINTER_INFO_6_64 { val dwStatus: DWORD }
    PRINTER_INFO_7W_TWIN = PRINTER_INFO_7W as PRINTER_INFO_7W_64 {
        ptr pszObjectGUID, val dwAction: DWORD,
    }
    PRINTER_INFO_8W_TWIN = PRINTER_INFO_8W as PRINTER_INFO_8W_64 { ptr pDevMode }
    PRINTER_INFO_9W_TWIN = PRINTER_INFO_9W as PRINTER_INFO_9W_64 { ptr pDevMode }
    PRINTER_INFO_STRESS_TWIN = PRINTER_INFO_STRESS as PRINTER_INFO_STRESS_64 {
        ptr pPrinterName, ptr pServerName, val cJobs: DWORD, val cTotalJobs: DWORD,
        val cTotalBytes: DWORD, val stUpTime: SYSTEMTIME, val MaxcRef: DWORD,
        val cTotalPagesPrinted: DWORD, val dwGetVersion: DWORD, val fFreeBuild: DWORD,
        val cSpooling: DWORD, val cMaxSpooling: DWORD, val cRef: DWORD,
        val cErrorOutOfPaper: DWORD, val cErrorNotReady: DWORD, val cJobError: DWORD,
        val dwNumberOfProcessors: DWORD, val dwProcessorType: DWORD,
        val dwHighPartTotalBytes: DWORD, val cChangeID: DWORD, val dwLastError: DWORD,
        val Status: DWORD, val cEnumerateNetworkPrinters: DWORD, val cAddNetPrinters: DWORD,
        val wProcessorArchitecture: USHORT, val wProcessorLevel: USHORT, val cRefIC: DWORD,
        val dwReserved2: DWORD, val dwReserved3: DWORD,
    }

    DRIVER_INFO_1W_TWIN = DRIVER_INFO_1W as DRIVER_INFO_1W_64 { ptr pName }
    DRIVER_INFO_2W_TWIN = DRIVER_INFO_2W as DRIVER_INFO_2W_64 {
        val cVersion: DWORD, ptr pName, ptr pEnvironment, ptr pDriverPath, ptr pDataFile,
        ptr pConfigFile,
    }
    DRIVER_INFO_3W_TWIN = DRIVER_INFO_3W as DRIVER_INFO_3W_64 {
        val cVersion: DWORD, ptr pName, ptr pEnvironment, ptr pDriverPath, ptr pDataFile,
        ptr pConfigFile, ptr pHelpFile, ptr pDependentFiles, ptr pMonitorName,
        ptr pDefaultDataType,
    }
    DRIVER_INFO_4W_TWIN = DRIVER_INFO_4W as DRIVER_INFO_4W_64 {
        val cVersion: DWORD, ptr pName, ptr pEnvironment, ptr pDriverPath, ptr pDataFile,
        ptr pConfigFile, ptr pHelpFile, ptr pDependentFiles, ptr pMonitorName,
        ptr pDefaultDataType, ptr pszzPreviousNames,
    }
    DRIVER_INFO_5W_TWIN = DRIVER_INFO_5W as DRIVER_INFO_5W_64 {
        val cVersion: DWORD, ptr pName, ptr pEnvironment, ptr pDriverPath, ptr pDataFile,
        ptr pConfigFile, val dwDriverAttributes: DWORD, val dwConfigVersion: DWORD,
        val dwDriverVersion: DWORD,
    }
    DRIVER_INFO_6W_TWIN = DRIVER_INFO_6W as DRIVER_INFO_6W_64 {
        val cVersion: DWORD, ptr pName, ptr pEnvironment, ptr pDriverPath, ptr pDataFile,
        ptr pConfigFile, ptr pHelpFile, ptr pDependentFiles, ptr pMonitorName,
        ptr pDefaultDataType, ptr pszzPreviousNames, val ftDriverDate: FILETIME,
        val dwlDriverVersion: DWORDLONG, ptr pszMfgName, ptr pszOEMUrl, ptr pszHardwareID,
        ptr pszProvider,
    }
    DRIVER_INFO_8W_TWIN = DRIVER_INFO_8W as DRIVER_INFO_8W_64 {
        val cVersion: DWORD, ptr pName, ptr pEnvironment, ptr pDriverPath, ptr pDataFile,
        ptr pConfigFile, ptr pHelpFile, ptr pDependentFiles, ptr pMonitorName,
        ptr pDefaultDataType, ptr pszzPreviousNames, val ftDriverDate: FILETIME,
        val dwlDriverVersion: DWORDLONG, ptr pszMfgName, ptr pszOEMUrl, ptr pszHardwareID,
        ptr pszProvider, ptr pszPrintProcessor, ptr pszVendorSetup, ptr pszzColorProfiles,
        ptr pszInfPath, val dwPrinterDriverAttributes: DWORD, ptr pszzCoreDriverDependencies,
        val ftMinInboxDriverVerDate: FILETIME, val dwlMinInboxDriverVerVersion: DWORDLONG,
    }

    ADDJOB_INFO_1W_TWIN = ADDJOB_INFO_1W as ADDJOB_INFO_1W_64 { ptr Path, val JobId: DWORD }
    JOB_INFO_1W_TWIN = JOB_INFO_1W as JOB_INFO_1W_64 {
        val JobId: DWORD, ptr pPrinterName, ptr pMachineName, ptr pUserName, ptr pDocument,
        ptr pDatatype, ptr pStatus, val Status: DWORD, val Priority: DWORD,
        val Position: DWORD, val TotalPages: DWORD, val PagesPrinted: DWORD,
        val Submitted: SYSTEMTIME,
    }
    JOB_INFO_2W_TWIN = JOB_INFO_2W as JOB_INFO_2W_64 {
        val JobId: DWORD, ptr pPrinterName, ptr pMachineName, ptr pUserName, ptr pDocument,
        ptr pNotifyName, ptr pDatatype, ptr pPrintProcessor, ptr pParameters,
        ptr pDriverName, ptr pDevMode, ptr pStatus, ptr pSecurityDescriptor,
        val Status: DWORD, val Priority: DWORD, val Position: DWORD, val StartTime: DWORD,
        val UntilTime: DWORD, val TotalPages: DWORD, val Size: DWORD,
        val Submitted: SYSTEMTIME, val Time: DWORD, val PagesPrinted: DWORD,
    }

    FORM_INFO_1W_TWIN = FORM_INFO_1W as FORM_INFO_1W_64 {
        val Flags: DWORD, ptr pName, val Size: SIZEL, val ImageableArea: RECTL,
    }
    FORM_INFO_2W_TWIN = FORM_INFO_2W as FORM_INFO_2W_64 {
        val Flags: DWORD, ptr pName, val Size: SIZEL, val ImageableArea: RECTL,
        ptr pKeyword, val StringType: DWORD, ptr pMuiDll, val dwResourceId: DWORD,
        ptr pDisplayName, val wLangId: LANGID,
    }

    MONITOR_INFO_1W_TWIN = MONITOR_INFO_1W as MONITOR_INFO_1W_64 { ptr pName }
    MONITOR_INFO_2W_TWIN = MONITOR_INFO_2W as MONITOR_INFO_2W_64 {
        ptr pName, ptr pEnvironment, ptr pDLLName,
    }
    PORT_INFO_1W_TWIN = PORT_INFO_1W as PORT_INFO_1W_64 { ptr pName }
    PORT_INFO_2W_TWIN = PORT_INFO_2W as PORT_INFO_2W_64 {
        ptr pPortName, ptr pMonitorName, ptr pDescription, val fPortType: DWORD,
        val Reserved: DWORD,
    }
    PRINTPROCESSOR_INFO_1W_TWIN = PRINTPROCESSOR_INFO_1W as PRINTPROCESSOR_INFO_1W_64 { ptr pName }
    DATATYPES_INFO_1W_TWIN = DATATYPES_INFO_1W as DATATYPES_INFO_1W_64 { ptr pName }
}

#[cfg(target_arch = "x86")]
extern "system" {
    fn IsWow64Process(hProcess: HANDLE, Wow64Process: PBOOL) -> BOOL;
}

/// 0 = not asked yet, 1 = native, 2 = WoW64. A process never changes which it is.
#[cfg(target_arch = "x86")]
static WOW64_STATE: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);

#[cfg(target_arch = "x86")]
unsafe fn UnderWow64() -> bool {
    use core::sync::atomic::Ordering;
    match WOW64_STATE.load(Ordering::Relaxed) {
        1 => false,
        2 => true,
        _ => {
            let mut bWow64: BOOL = FALSE as BOOL;
            // NtCurrentProcess(). A failed query is treated as native: that is the layout the
            // unmarshalling assumed before this existed.
            let bOk = IsWow64Process(-1isize as HANDLE, &mut bWow64) != 0 && bWow64 != 0;
            WOW64_STATE.store(if bOk { 2 } else { 1 }, Ordering::Relaxed);
            bOk
        }
    }
}

/// The twin for the table the caller is unmarshalling with, by the table's identity.
#[cfg(target_arch = "x86")]
unsafe fn TwinOf(pInfo: *const MARSHALLING_INFO) -> Option<&'static Twin> {
    macro_rules! by_table {
        ($($table:ident => $twin:ident),* $(,)?) => {
            $(if pInfo == (&raw const $table.pInfo).cast::<MARSHALLING_INFO>() {
                return Some(&$twin);
            })*
        };
    }
    by_table! {
        PrinterInfo1Marshalling => PRINTER_INFO_1W_TWIN,
        PrinterInfo2Marshalling => PRINTER_INFO_2W_TWIN,
        PrinterInfo3Marshalling => PRINTER_INFO_3_TWIN,
        PrinterInfo4Marshalling => PRINTER_INFO_4W_TWIN,
        PrinterInfo5Marshalling => PRINTER_INFO_5W_TWIN,
        PrinterInfo6Marshalling => PRINTER_INFO_6_TWIN,
        PrinterInfo7Marshalling => PRINTER_INFO_7W_TWIN,
        PrinterInfo8Marshalling => PRINTER_INFO_8W_TWIN,
        PrinterInfo9Marshalling => PRINTER_INFO_9W_TWIN,
        PrinterInfoStressMarshalling => PRINTER_INFO_STRESS_TWIN,
        PrinterDriver1Marshalling => DRIVER_INFO_1W_TWIN,
        PrinterDriver2Marshalling => DRIVER_INFO_2W_TWIN,
        PrinterDriver3Marshalling => DRIVER_INFO_3W_TWIN,
        PrinterDriver4Marshalling => DRIVER_INFO_4W_TWIN,
        PrinterDriver5Marshalling => DRIVER_INFO_5W_TWIN,
        PrinterDriver6Marshalling => DRIVER_INFO_6W_TWIN,
        PrinterDriver8Marshalling => DRIVER_INFO_8W_TWIN,
        AddJobInfo1Marshalling => ADDJOB_INFO_1W_TWIN,
        JobInfo1Marshalling => JOB_INFO_1W_TWIN,
        JobInfo2Marshalling => JOB_INFO_2W_TWIN,
        FormInfo1Marshalling => FORM_INFO_1W_TWIN,
        FormInfo2Marshalling => FORM_INFO_2W_TWIN,
        MonitorInfo1Marshalling => MONITOR_INFO_1W_TWIN,
        MonitorInfo2Marshalling => MONITOR_INFO_2W_TWIN,
        PortInfo1Marshalling => PORT_INFO_1W_TWIN,
        PortInfo2Marshalling => PORT_INFO_2W_TWIN,
        PrintProcessorInfo1Marshalling => PRINTPROCESSOR_INFO_1W_TWIN,
        DatatypesInfo1Marshalling => DATATYPES_INFO_1W_TWIN,
    }
    None
}

/// The table and its twin describe one structure: same size, and the table's adjusted fields
/// are exactly the twin's pointers. Catches a twin paired with the wrong table.
#[cfg(target_arch = "x86")]
unsafe fn TwinMatchesTable(twin: &Twin, mut pInfo: *const MARSHALLING_INFO, cbStructureSize: DWORD) -> bool {
    if twin.size32 != cbStructureSize {
        return false;
    }
    let mut cAdjusted = 0usize;
    while (*pInfo).dwOffset != MAXDWORD as DWORD {
        if (*pInfo).bAdjustAddress != 0 {
            if !twin.fields.iter().any(|f| f.pointer && f.at == (*pInfo).dwOffset) {
                return false;
            }
            cAdjusted += 1;
        }
        pInfo = pInfo.offset(1);
    }
    cAdjusted == twin.fields.iter().filter(|f| f.pointer).count()
}

/// Rewrite `cElements` structures the native spooler flattened at `pStructures` into this
/// build's layout, in place, ahead of `MarshallUpStructure`. Each element moves down from
/// `i * size64` to `i * size32` and its strings stay put, so its offsets grow by the
/// difference. Every field lands at or below where it is read from, and fields are visited in
/// ascending order, so no field is overwritten before it has been read. A no-op outside WoW64.
#[cfg(target_arch = "x86")]
pub(crate) unsafe fn NarrowSpoolerStructures(
    cbSize: DWORD,
    pStructures: PVOID,
    cElements: DWORD,
    pInfo: *const MARSHALLING_INFO,
    cbStructureSize: DWORD,
) -> BOOL {
    if cElements == 0 || !UnderWow64() {
        return TRUE as BOOL;
    }
    let twin = match TwinOf(pInfo) {
        Some(t) if TwinMatchesTable(t, pInfo, cbStructureSize) => t,
        _ => {
            ERR!("WoW64: no 64-bit layout for a {}-byte spooler structure, refusing to unmarshall it\n", cbStructureSize);
            SetLastError(ERROR_INVALID_DATA as DWORD);
            return FALSE as BOOL;
        }
    };
    let cbNeeded = (cElements as u64) * (twin.size64 as u64);
    if cbNeeded > cbSize as u64 {
        SetLastError(ERROR_INVALID_DATA as DWORD);
        return FALSE as BOOL;
    }
    let pBase = pStructures as *mut u8;
    for i in 0..cElements {
        let pFrom = pBase.add((i * twin.size64) as usize);
        let pTo = pBase.add((i * twin.size32) as usize);
        let dwShift = i * (twin.size64 - twin.size32);
        for f in twin.fields {
            if f.pointer {
                let qwOffset = (pFrom.add(f.from as usize) as *const u64).read_unaligned();
                let dwOffset = if qwOffset == 0 {
                    0
                } else {
                    match u32::try_from(qwOffset).ok().and_then(|d| d.checked_add(dwShift)) {
                        Some(d) => d,
                        None => {
                            SetLastError(ERROR_INVALID_DATA as DWORD);
                            return FALSE as BOOL;
                        }
                    }
                };
                (pTo.add(f.at as usize) as *mut u32).write_unaligned(dwOffset);
            } else {
                core::ptr::copy(pFrom.add(f.from as usize), pTo.add(f.at as usize), f.size as usize);
            }
        }
    }
    TRUE as BOOL
}
