//! kdbg/tables.rs — ComponentTable + KdbDebuggerCommands data tables and the
//! enum-to-string helpers that used to live in `ntoskrnl/kdbg/kdbg_rust_shims.c`.
//!
//! Translated from C 2026-05-07. The original C tables were silently
//! link-dead because the kdb_cli.rs `extern "C" { static mut FOO: [T; 0]; }`
//! declarations made any indexing fold to an unconditional bounds-check
//! panic — see the deleted block comment in kdbg_rust_shims.c history.
//! Properly sizing the arrays in Rust unblocks the corresponding KDBG
//! commands (filter, help, dispatch).

use super::kdb_cli::{KDB_NAMED_OBJECT, _KDB_DEBUGGER_COMMAND};
use super::types::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{PCSTR, PSTR};

// ---------------------------------------------------------------------------
// ComponentTable — 156 entries (1 special + 155 KD_DEBUG_PRINT_FILTER).
// Used by KdbpCmdFilter to map debug-channel names to DPFLTR_* IDs.
// ---------------------------------------------------------------------------

#[no_mangle]
pub static mut ComponentTable: [KDB_NAMED_OBJECT; 156] = [
    KDB_NAMED_OBJECT { Name: b"WIN2000\0".as_ptr() as PCSTR, Id: u32::MAX },
    KDB_NAMED_OBJECT { Name: b"DEFAULT\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_DEFAULT_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SYSTEM\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SYSTEM_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SMSS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SMSS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SETUP\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SETUP_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"NTFS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_NTFS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"FSTUB\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_FSTUB_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"CRASHDUMP\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_CRASHDUMP_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"CDAUDIO\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_CDAUDIO_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"CDROM\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_CDROM_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"CLASSPNP\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_CLASSPNP_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"DISK\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_DISK_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"REDBOOK\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_REDBOOK_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"STORPROP\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_STORPROP_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SCSIPORT\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SCSIPORT_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SCSIMINIPORT\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SCSIMINIPORT_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"CONFIG\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_CONFIG_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"I8042PRT\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_I8042PRT_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SERMOUSE\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SERMOUSE_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"LSERMOUS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_LSERMOUS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"KBDHID\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_KBDHID_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"MOUHID\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_MOUHID_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"KBDCLASS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_KBDCLASS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"MOUCLASS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_MOUCLASS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"TWOTRACK\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_TWOTRACK_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"WMILIB\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_WMILIB_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"ACPI\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_ACPI_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"AMLI\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_AMLI_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"HALIA64\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_HALIA64_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"VIDEO\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_VIDEO_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SVCHOST\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SVCHOST_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"VIDEOPRT\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_VIDEOPRT_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"TCPIP\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_TCPIP_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"DMSYNTH\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_DMSYNTH_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"NTOSPNP\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_NTOSPNP_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"FASTFAT\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_FASTFAT_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SAMSS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SAMSS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"PNPMGR\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_PNPMGR_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"NETAPI\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_NETAPI_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SCSERVER\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SCSERVER_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SCCLIENT\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SCCLIENT_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SERIAL\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SERIAL_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SERENUM\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SERENUM_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"UHCD\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_UHCD_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"RPCPROXY\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_RPCPROXY_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"AUTOCHK\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_AUTOCHK_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"DCOMSS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_DCOMSS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"UNIMODEM\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_UNIMODEM_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SIS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SIS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"FLTMGR\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_FLTMGR_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"WMICORE\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_WMICORE_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"BURNENG\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_BURNENG_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"IMAPI\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_IMAPI_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SXS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SXS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"FUSION\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_FUSION_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"IDLETASK\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_IDLETASK_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SOFTPCI\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SOFTPCI_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"TAPE\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_TAPE_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"MCHGR\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_MCHGR_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"IDEP\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_IDEP_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"PCIIDE\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_PCIIDE_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"FLOPPY\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_FLOPPY_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"FDC\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_FDC_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"TERMSRV\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_TERMSRV_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"W32TIME\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_W32TIME_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"PREFETCHER\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_PREFETCHER_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"RSFILTER\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_RSFILTER_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"FCPORT\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_FCPORT_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"PCI\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_PCI_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"DMIO\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_DMIO_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"DMCONFIG\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_DMCONFIG_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"DMADMIN\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_DMADMIN_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"WSOCKTRANSPORT\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_WSOCKTRANSPORT_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"VSS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_VSS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"PNPMEM\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_PNPMEM_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"PROCESSOR\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_PROCESSOR_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"DMSERVER\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_DMSERVER_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SR\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SR_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"INFINIBAND\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_INFINIBAND_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"IHVDRIVER\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_IHVDRIVER_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"IHVVIDEO\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_IHVVIDEO_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"IHVAUDIO\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_IHVAUDIO_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"IHVNETWORK\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_IHVNETWORK_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"IHVSTREAMING\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_IHVSTREAMING_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"IHVBUS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_IHVBUS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"HPS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_HPS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"RTLTHREADPOOL\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_RTLTHREADPOOL_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"LDR\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_LDR_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"TCPIP6\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_TCPIP6_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"ISAPNP\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_ISAPNP_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SHPC\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SHPC_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"STORPORT\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_STORPORT_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"STORMINIPORT\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_STORMINIPORT_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"PRINTSPOOLER\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_PRINTSPOOLER_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"VSSDYNDISK\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_VSSDYNDISK_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"VERIFIER\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_VERIFIER_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"VDS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_VDS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"VDSBAS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_VDSBAS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"VDSDYN\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_VDSDYN_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"VDSDYNDR\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_VDSDYNDR_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"VDSLDR\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_VDSLDR_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"VDSUTIL\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_VDSUTIL_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"DFRGIFC\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_DFRGIFC_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"MM\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_MM_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"DFSC\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_DFSC_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"WOW64\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_WOW64_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"ALPC\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_ALPC_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"WDI\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_WDI_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"PERFLIB\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_PERFLIB_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"KTM\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_KTM_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"IOSTRESS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_IOSTRESS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"HEAP\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_HEAP_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"WHEA\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_WHEA_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"USERGDI\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_USERGDI_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"MMCSS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_MMCSS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"TPM\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_TPM_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"THREADORDER\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_THREADORDER_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"ENVIRON\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_ENVIRON_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"EMS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_EMS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"WDT\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_WDT_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"FVEVOL\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_FVEVOL_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"NDIS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_NDIS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"NVCTRACE\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_NVCTRACE_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"LUAFV\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_LUAFV_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"APPCOMPAT\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_APPCOMPAT_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"USBSTOR\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_USBSTOR_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SBP2PORT\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SBP2PORT_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"COVERAGE\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_COVERAGE_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"CACHEMGR\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_CACHEMGR_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"MOUNTMGR\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_MOUNTMGR_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"CFR\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_CFR_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"TXF\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_TXF_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"KSECDD\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_KSECDD_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"FLTREGRESS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_FLTREGRESS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"MPIO\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_MPIO_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"MSDSM\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_MSDSM_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"UDFS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_UDFS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"PSHED\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_PSHED_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"STORVSP\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_STORVSP_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"LSASS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_LSASS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SSPICLI\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SSPICLI_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"CNG\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_CNG_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"EXFAT\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_EXFAT_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"FILETRACE\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_FILETRACE_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"XSAVE\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_XSAVE_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"SE\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_SE_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"DRIVEEXTENDER\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_DRIVEEXTENDER_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"POWER\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_POWER_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"CRASHDUMPXHCI\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_CRASHDUMPXHCI_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"GPIO\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_GPIO_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"REFS\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_REFS_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"WER\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_WER_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"CAPIMG\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_CAPIMG_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"VPCI\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_VPCI_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"STORAGECLASSMEMORY\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_STORAGECLASSMEMORY_ID as ULONG },
    KDB_NAMED_OBJECT { Name: b"FSLIB\0".as_ptr() as PCSTR, Id: ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_FSLIB_ID as ULONG },
];

// ---------------------------------------------------------------------------
// KdbDebuggerCommands — KDBG `kdb:>` command dispatch table.
//
// Group separators have NULL Name+Syntax+Fn; only Help is the section title.
// Function pointers all live in `super::kdb_cli` as `#[no_mangle] extern "C"`
// exports. tss/disasm have arch-stubbed Rust implementations that print
// "not supported" on non-x86 — included unconditionally so the table layout
// is the same on every architecture (KdbpCmdHelp iterates by index).
// ---------------------------------------------------------------------------

use super::kdb_cli::{
    KdbpCmdEvalExpression, KdbpCmdDisassembleX, KdbpCmdRegs, KdbpCmdBackTrace,
    KdbpCmdContinue, KdbpCmdStep, KdbpCmdBreakPointList,
    KdbpCmdEnableDisableClearBreakPoint, KdbpCmdBreakPoint, KdbpCmdThread,
    KdbpCmdProc, KdbpCmdMod, KdbpCmdGdtLdtIdt, KdbpCmdPcr, KdbpCmdTss,
    KdbpCmdBugCheck, KdbpCmdReboot, KdbpCmdFilter, KdbpCmdSet, KdbpCmdHelp,
    KdbpCmdDmesg, KdbpCmdPrintStruct,
};

extern "C" {
    fn ExpKdbgExtPool(Argc: ULONG, Argv: *mut PCHAR) -> BOOLEAN;
    fn ExpKdbgExtPoolUsed(Argc: ULONG, Argv: *mut PCHAR) -> BOOLEAN;
    fn ExpKdbgExtPoolFind(Argc: ULONG, Argv: *mut PCHAR) -> BOOLEAN;
    fn ExpKdbgExtFileCache(Argc: ULONG, Argv: *mut PCHAR) -> BOOLEAN;
    fn ExpKdbgExtDefWrites(Argc: ULONG, Argv: *mut PCHAR) -> BOOLEAN;
    fn ExpKdbgExtIrpFind(Argc: ULONG, Argv: *mut PCHAR) -> BOOLEAN;
    fn ExpKdbgExtHandle(Argc: ULONG, Argv: *mut PCHAR) -> BOOLEAN;
}

macro_rules! cmd {
    ($name:expr, $syn:expr, $help:expr, $fn_:expr) => {
        _KDB_DEBUGGER_COMMAND {
            Name: $name as PCHAR,
            Syntax: $syn as PCHAR,
            Help: $help as PCHAR,
            Fn: Some($fn_),
        }
    };
}

const fn null_pchar() -> PCHAR { core::ptr::null_mut() }

const fn section(title: *const u8) -> _KDB_DEBUGGER_COMMAND {
    _KDB_DEBUGGER_COMMAND {
        Name: null_pchar(),
        Syntax: null_pchar(),
        Help: title as PCHAR,
        Fn: None,
    }
}

#[no_mangle]
pub static mut KdbDebuggerCommands: [_KDB_DEBUGGER_COMMAND; 45] = [
    /* Data */
    section(b"Data\0".as_ptr()),
    cmd!(b"?\0".as_ptr(),       b"? expression\0".as_ptr(),                                     b"Evaluate expression.\0".as_ptr(),                                                       KdbpCmdEvalExpression),
    cmd!(b"disasm\0".as_ptr(),  b"disasm [address] [L count]\0".as_ptr(),                       b"Disassemble count instructions at address.\0".as_ptr(),                                  KdbpCmdDisassembleX),
    cmd!(b"x\0".as_ptr(),       b"x [address] [L count]\0".as_ptr(),                            b"Display count dwords, starting at address.\0".as_ptr(),                                  KdbpCmdDisassembleX),
    cmd!(b"regs\0".as_ptr(),    b"regs\0".as_ptr(),                                             b"Display general purpose registers.\0".as_ptr(),                                          KdbpCmdRegs),
    cmd!(b"cregs\0".as_ptr(),   b"cregs\0".as_ptr(),                                            b"Display control, descriptor table and task segment registers.\0".as_ptr(),               KdbpCmdRegs),
    cmd!(b"sregs\0".as_ptr(),   b"sregs\0".as_ptr(),                                            b"Display status registers.\0".as_ptr(),                                                   KdbpCmdRegs),
    cmd!(b"dregs\0".as_ptr(),   b"dregs\0".as_ptr(),                                            b"Display debug registers.\0".as_ptr(),                                                    KdbpCmdRegs),
    cmd!(b"bt\0".as_ptr(),      b"bt [*frameaddr|thread id]\0".as_ptr(),                        b"Prints current backtrace or from given frame address.\0".as_ptr(),                       KdbpCmdBackTrace),
    cmd!(b"dt\0".as_ptr(),      b"dt [mod] [type] [addr]\0".as_ptr(),                           b"Print a struct. The address is optional.\0".as_ptr(),                                    KdbpCmdPrintStruct),
    /* Flow control */
    section(b"Flow control\0".as_ptr()),
    cmd!(b"cont\0".as_ptr(),    b"cont\0".as_ptr(),                                             b"Continue execution (leave debugger).\0".as_ptr(),                                        KdbpCmdContinue),
    cmd!(b"step\0".as_ptr(),    b"step [count]\0".as_ptr(),                                     b"Execute single instructions, stepping into interrupts.\0".as_ptr(),                      KdbpCmdStep),
    cmd!(b"next\0".as_ptr(),    b"next [count]\0".as_ptr(),                                     b"Execute single instructions, skipping calls and reps.\0".as_ptr(),                       KdbpCmdStep),
    cmd!(b"bl\0".as_ptr(),      b"bl\0".as_ptr(),                                               b"List breakpoints.\0".as_ptr(),                                                           KdbpCmdBreakPointList),
    cmd!(b"be\0".as_ptr(),      b"be [breakpoint]\0".as_ptr(),                                  b"Enable breakpoint.\0".as_ptr(),                                                          KdbpCmdEnableDisableClearBreakPoint),
    cmd!(b"bd\0".as_ptr(),      b"bd [breakpoint]\0".as_ptr(),                                  b"Disable breakpoint.\0".as_ptr(),                                                         KdbpCmdEnableDisableClearBreakPoint),
    cmd!(b"bc\0".as_ptr(),      b"bc [breakpoint]\0".as_ptr(),                                  b"Clear breakpoint.\0".as_ptr(),                                                           KdbpCmdEnableDisableClearBreakPoint),
    cmd!(b"bpx\0".as_ptr(),     b"bpx [address] [IF condition]\0".as_ptr(),                     b"Set software execution breakpoint at address.\0".as_ptr(),                               KdbpCmdBreakPoint),
    cmd!(b"bpm\0".as_ptr(),     b"bpm [r|w|rw|x] [byte|word|dword] [address] [IF condition]\0".as_ptr(), b"Set memory breakpoint at address.\0".as_ptr(),                                          KdbpCmdBreakPoint),
    /* Process/Thread */
    section(b"Process/Thread\0".as_ptr()),
    cmd!(b"thread\0".as_ptr(),  b"thread [list[ pid]|[attach ]tid]\0".as_ptr(),                 b"List threads in current or specified process, display thread with given id or attach to thread.\0".as_ptr(), KdbpCmdThread),
    cmd!(b"proc\0".as_ptr(),    b"proc [list|[attach ]pid]\0".as_ptr(),                         b"List processes, display process with given id or attach to process.\0".as_ptr(),         KdbpCmdProc),
    /* System info */
    section(b"System info\0".as_ptr()),
    cmd!(b"mod\0".as_ptr(),     b"mod [address]\0".as_ptr(),                                    b"List all modules or the one containing address.\0".as_ptr(),                             KdbpCmdMod),
    cmd!(b"gdt\0".as_ptr(),     b"gdt\0".as_ptr(),                                              b"Display the global descriptor table.\0".as_ptr(),                                        KdbpCmdGdtLdtIdt),
    cmd!(b"ldt\0".as_ptr(),     b"ldt\0".as_ptr(),                                              b"Display the local descriptor table.\0".as_ptr(),                                         KdbpCmdGdtLdtIdt),
    cmd!(b"idt\0".as_ptr(),     b"idt\0".as_ptr(),                                              b"Display the interrupt descriptor table.\0".as_ptr(),                                     KdbpCmdGdtLdtIdt),
    cmd!(b"pcr\0".as_ptr(),     b"pcr\0".as_ptr(),                                              b"Display the processor control region.\0".as_ptr(),                                       KdbpCmdPcr),
    cmd!(b"tss\0".as_ptr(),     b"tss [selector|*descaddr]\0".as_ptr(),                         b"Display the current task state segment, or the one specified by its selector number or descriptor address.\0".as_ptr(), KdbpCmdTss),
    /* Others */
    section(b"Others\0".as_ptr()),
    cmd!(b"bugcheck\0".as_ptr(), b"bugcheck\0".as_ptr(),                                         b"Bugchecks the system.\0".as_ptr(),                                                       KdbpCmdBugCheck),
    cmd!(b"reboot\0".as_ptr(),  b"reboot\0".as_ptr(),                                           b"Reboots the system.\0".as_ptr(),                                                         KdbpCmdReboot),
    cmd!(b"filter\0".as_ptr(),  b"filter [error|warning|trace|info|level]+|-[componentname|default]\0".as_ptr(), b"Enable/disable debug channels.\0".as_ptr(),                                          KdbpCmdFilter),
    cmd!(b"set\0".as_ptr(),     b"set [var] [value]\0".as_ptr(),                                b"Sets var to value or displays value of var.\0".as_ptr(),                                 KdbpCmdSet),
    cmd!(b"dmesg\0".as_ptr(),   b"dmesg\0".as_ptr(),                                            b"Display debug messages on screen, with navigation on pages.\0".as_ptr(),                 KdbpCmdDmesg),
    cmd!(b"kmsg\0".as_ptr(),    b"kmsg\0".as_ptr(),                                             b"Kernel dmesg. Alias for dmesg.\0".as_ptr(),                                              KdbpCmdDmesg),
    cmd!(b"help\0".as_ptr(),    b"help\0".as_ptr(),                                             b"Display help screen.\0".as_ptr(),                                                        KdbpCmdHelp),
    cmd!(b"!pool\0".as_ptr(),       b"!pool [Address [Flags]]\0".as_ptr(),                      b"Display information about pool allocations.\0".as_ptr(),                                 ExpKdbgExtPool),
    cmd!(b"!poolused\0".as_ptr(),   b"!poolused [Flags [Tag]]\0".as_ptr(),                      b"Display pool usage.\0".as_ptr(),                                                         ExpKdbgExtPoolUsed),
    cmd!(b"!poolfind\0".as_ptr(),   b"!poolfind Tag [Pool]\0".as_ptr(),                         b"Search for pool tag allocations.\0".as_ptr(),                                            ExpKdbgExtPoolFind),
    cmd!(b"!filecache\0".as_ptr(),  b"!filecache\0".as_ptr(),                                   b"Display cache usage.\0".as_ptr(),                                                        ExpKdbgExtFileCache),
    cmd!(b"!defwrites\0".as_ptr(),  b"!defwrites\0".as_ptr(),                                   b"Display cache write values.\0".as_ptr(),                                                 ExpKdbgExtDefWrites),
    cmd!(b"!irpfind\0".as_ptr(),    b"!irpfind [Pool [startaddress [criteria data]]]\0".as_ptr(), b"Lists IRPs potentially matching criteria.\0".as_ptr(),                                  ExpKdbgExtIrpFind),
    cmd!(b"!handle\0".as_ptr(),     b"!handle [Handle]\0".as_ptr(),                             b"Displays info about handles.\0".as_ptr(),                                                ExpKdbgExtHandle),
];

// ---------------------------------------------------------------------------
// Enum-to-string helpers — translated from kdbg_rust_shims.c.
// Kept as `extern "C"` exports so existing kdb_cli.rs `extern "C" { fn ... }`
// declarations resolve transparently once the C versions are deleted.
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn KDB_ACCESS_TYPE_TO_STRING(t: KDB_ACCESS_TYPE) -> PCHAR {
    let s: &[u8] = match t {
        KdbAccessRead => b"read\0",
        KdbAccessWrite => b"write\0",
        KdbAccessReadWrite => b"rdwr\0",
        KdbAccessExec => b"exec\0",
    };
    s.as_ptr() as PCHAR
}

#[no_mangle]
pub unsafe extern "C" fn KDB_ENTER_CONDITION_TO_STRING(cond: KDB_ENTER_CONDITION) -> PCHAR {
    let s: &[u8] = match cond {
        KdbDoNotEnter => b"never\0",
        KdbEnterAlways => b"always\0",
        KdbEnterFromKmode => b"kmode\0",
        KdbEnterFromUmode => b"umode\0",
    };
    s.as_ptr() as PCHAR
}

/// `state` is `NPX_STATE_LOADED` (0) or `NPX_STATE_NOT_LOADED` (0x10) per psfuncs.h.
#[no_mangle]
pub unsafe extern "C" fn NPX_STATE_TO_STRING(state: u32) -> PSTR {
    let s: &[u8] = match state {
        0 => b"Loaded\0",
        0x10 => b"Not loaded\0",
        _ => b"Unknown\0",
    };
    s.as_ptr() as PSTR
}

// ============================================================================
// kdbg globals — ownership moved from kdbg_rust_shims.c. kdb_cli.rs declares
// each via `extern "C" { static mut X: T; }` so storage has to live on this
// side; bool/u32/PCHAR types are layout-trivial across all 3 arches.
// ============================================================================

/// `PKDBG_CLI_ROUTINE` — file-private typedef in the original C source.
/// Re-rendered as a function pointer; the slot table is fixed-size 10.
type PKDBG_CLI_ROUTINE = Option<
    unsafe extern "system" fn(
        Command: *mut core::ffi::c_char,
        Argc: ntkrnl_sys::base::ULONG,
        Argv: *mut *mut core::ffi::c_char,
    ) -> ntkrnl_sys::base::BOOLEAN,
>;

/// CLI callback slots — fixed-size 10, registered at runtime.
#[no_mangle]
pub static mut KdbCliCallbacks: [PKDBG_CLI_ROUTINE; 10] = [None; 10];

#[no_mangle]
pub static mut KdbUseIntelSyntax: ntkrnl_sys::base::BOOLEAN = 1; // TRUE
#[no_mangle]
pub static mut KdbBreakOnModuleLoad: ntkrnl_sys::base::BOOLEAN = 0;
#[no_mangle]
pub static mut KdbNumberOfRowsPrinted: ntkrnl_sys::base::ULONG = 0;
#[no_mangle]
pub static mut KdbNumberOfColsPrinted: ntkrnl_sys::base::ULONG = 0;
#[no_mangle]
pub static mut KdbOutputAborted: ntkrnl_sys::base::BOOLEAN = 0;
#[no_mangle]
pub static mut KdbRepeatLastCommand: ntkrnl_sys::base::BOOLEAN = 0;

/// `KdbInitFileBuffer` and `KdbpBugCheckRequested` — defined here in Rust
/// since the original C kdb_cli.c is excluded from the build.
#[no_mangle]
pub static mut KdbInitFileBuffer: *mut core::ffi::c_char = core::ptr::null_mut();
#[no_mangle]
pub static mut KdbpBugCheckRequested: ntkrnl_sys::base::BOOLEAN = 0;

/// Dmesg ring buffer state — read+written from kdb_cli.rs serial-output paths.
#[no_mangle]
pub static KdpDmesgBufferSize: ntkrnl_sys::base::ULONG = 128 * 1024;
#[no_mangle]
pub static mut KdpDmesgBuffer: *mut core::ffi::c_char = core::ptr::null_mut();
#[no_mangle]
pub static mut KdpDmesgCurrentPosition: ntkrnl_sys::base::ULONG = 0;
#[no_mangle]
pub static mut KdpDmesgFreeBytes: ntkrnl_sys::base::ULONG = 0;
#[no_mangle]
pub static mut KdbDmesgTotalWritten: ntkrnl_sys::base::ULONG = 0;
#[no_mangle]
pub static mut KdbpIsInDmesgMode: ntkrnl_sys::base::BOOLEAN = 0;

/// `KSPIN_LOCK` is a single ULONG_PTR on all 3 arches; zero-init by convention.
#[no_mangle]
pub static mut KdpDmesgLogSpinLock: ntkrnl_sys::base::ULONG_PTR = 0;
