//! Kernel bugcheck (stop) codes — GENERATED, do not edit by hand.
//!
//! Regenerate with `python3 tools/gen_bugcodes.py` after touching
//! `sdk/include/reactos/mc/bugcodes.mc`.
//!
//! Pass these to `KeBugCheck`/`KeBugCheckEx` instead of a numeric literal. A
//! hand-written number is how 61 translated stop codes ended up naming the
//! wrong subsystem (field manual §9.113); `tools/scan_bugcheck_codes.py` keeps
//! new ones out.
//!
//! Source: sdk/include/reactos/mc/bugcodes.mc (187 stop codes + 16
//! bugcheck-screen message strings, which keep their message-compiler severity
//! bits), cross-verified against
//! sdk/lib/rust/reactos-link/committed-headers/bugcodes.h.

#![allow(dead_code)]

use crate::ddk::ULONG;

pub const UNDEFINED_BUG_CODE                                       : ULONG = 0x0;
pub const APC_INDEX_MISMATCH                                       : ULONG = 0x1;
pub const DEVICE_QUEUE_NOT_BUSY                                    : ULONG = 0x2;
pub const INVALID_AFFINITY_SET                                     : ULONG = 0x3;
pub const INVALID_DATA_ACCESS_TRAP                                 : ULONG = 0x4;
pub const INVALID_PROCESS_ATTACH_ATTEMPT                           : ULONG = 0x5;
pub const INVALID_PROCESS_DETACH_ATTEMPT                           : ULONG = 0x6;
pub const INVALID_SOFTWARE_INTERRUPT                               : ULONG = 0x7;
pub const IRQL_NOT_DISPATCH_LEVEL                                  : ULONG = 0x8;
pub const IRQL_NOT_GREATER_OR_EQUAL                                : ULONG = 0x9;
pub const IRQL_NOT_LESS_OR_EQUAL                                   : ULONG = 0xA;
pub const NO_EXCEPTION_HANDLING_SUPPORT                            : ULONG = 0xB;
pub const MAXIMUM_WAIT_OBJECTS_EXCEEDED                            : ULONG = 0xC;
pub const MUTEX_LEVEL_NUMBER_VIOLATION                             : ULONG = 0xD;
pub const NO_USER_MODE_CONTEXT                                     : ULONG = 0xE;
pub const SPIN_LOCK_ALREADY_OWNED                                  : ULONG = 0xF;
pub const SPIN_LOCK_NOT_OWNED                                      : ULONG = 0x10;
pub const THREAD_NOT_MUTEX_OWNER                                   : ULONG = 0x11;
pub const TRAP_CAUSE_UNKNOWN                                       : ULONG = 0x12;
pub const EMPTY_THREAD_REAPER_LIST                                 : ULONG = 0x13;
pub const CREATE_DELETE_LOCK_NOT_LOCKED                            : ULONG = 0x14;
pub const LAST_CHANCE_CALLED_FROM_KMODE                            : ULONG = 0x15;
pub const CID_HANDLE_CREATION                                      : ULONG = 0x16;
pub const CID_HANDLE_DELETION                                      : ULONG = 0x17;
pub const REFERENCE_BY_POINTER                                     : ULONG = 0x18;
pub const BAD_POOL_HEADER                                          : ULONG = 0x19;
pub const MEMORY_MANAGEMENT                                        : ULONG = 0x1A;
pub const PFN_SHARE_COUNT                                          : ULONG = 0x1B;
pub const PFN_REFERENCE_COUNT                                      : ULONG = 0x1C;
pub const NO_SPINLOCK_AVAILABLE                                    : ULONG = 0x1D;
pub const KMODE_EXCEPTION_NOT_HANDLED                              : ULONG = 0x1E;
pub const SHARED_RESOURCE_CONV_ERROR                               : ULONG = 0x1F;
pub const KERNEL_APC_PENDING_DURING_EXIT                           : ULONG = 0x20;
pub const QUOTA_UNDERFLOW                                          : ULONG = 0x21;
pub const FILE_SYSTEM                                              : ULONG = 0x22;
pub const FAT_FILE_SYSTEM                                          : ULONG = 0x23;
pub const NTFS_FILE_SYSTEM                                         : ULONG = 0x24;
pub const NPFS_FILE_SYSTEM                                         : ULONG = 0x25;
pub const CDFS_FILE_SYSTEM                                         : ULONG = 0x26;
pub const RDR_FILE_SYSTEM                                          : ULONG = 0x27;
pub const CORRUPT_ACCESS_TOKEN                                     : ULONG = 0x28;
pub const SECURITY_SYSTEM                                          : ULONG = 0x29;
pub const INCONSISTENT_IRP                                         : ULONG = 0x2A;
pub const PANIC_STACK_SWITCH                                       : ULONG = 0x2B;
pub const PORT_DRIVER_INTERNAL                                     : ULONG = 0x2C;
pub const SCSI_DISK_DRIVER_INTERNAL                                : ULONG = 0x2D;
pub const DATA_BUS_ERROR                                           : ULONG = 0x2E;
pub const INSTRUCTION_BUS_ERROR                                    : ULONG = 0x2F;
pub const SET_OF_INVALID_CONTEXT                                   : ULONG = 0x30;
pub const PHASE0_INITIALIZATION_FAILED                             : ULONG = 0x31;
pub const PHASE1_INITIALIZATION_FAILED                             : ULONG = 0x32;
pub const UNEXPECTED_INITIALIZATION_CALL                           : ULONG = 0x33;
pub const CACHE_MANAGER                                            : ULONG = 0x34;
pub const NO_MORE_IRP_STACK_LOCATIONS                              : ULONG = 0x35;
pub const DEVICE_REFERENCE_COUNT_NOT_ZERO                          : ULONG = 0x36;
pub const FLOPPY_INTERNAL_ERROR                                    : ULONG = 0x37;
pub const SERIAL_DRIVER_INTERNAL                                   : ULONG = 0x38;
pub const SYSTEM_EXIT_OWNED_MUTEX                                  : ULONG = 0x39;
pub const MULTIPROCESSOR_CONFIGURATION_NOT_SUPPORTED               : ULONG = 0x3E;
pub const NO_MORE_SYSTEM_PTES                                      : ULONG = 0x3F;
pub const TARGET_MDL_TOO_SMALL                                     : ULONG = 0x40;
pub const MUST_SUCCEED_POOL_EMPTY                                  : ULONG = 0x41;
pub const ATDISK_DRIVER_INTERNAL                                   : ULONG = 0x42;
pub const MULTIPLE_IRP_COMPLETE_REQUESTS                           : ULONG = 0x44;
pub const INSUFFICIENT_SYSTEM_MAP_REGS                             : ULONG = 0x45;
pub const CANCEL_STATE_IN_COMPLETED_IRP                            : ULONG = 0x48;
pub const PAGE_FAULT_WITH_INTERRUPTS_OFF                           : ULONG = 0x49;
pub const IRQL_GT_ZERO_AT_SYSTEM_SERVICE                           : ULONG = 0x4A;
pub const STREAMS_INTERNAL_ERROR                                   : ULONG = 0x4B;
pub const FATAL_UNHANDLED_HARD_ERROR                               : ULONG = 0x4C;
pub const NO_PAGES_AVAILABLE                                       : ULONG = 0x4D;
pub const PFN_LIST_CORRUPT                                         : ULONG = 0x4E;
pub const NDIS_INTERNAL_ERROR                                      : ULONG = 0x4F;
pub const PAGE_FAULT_IN_NONPAGED_AREA                              : ULONG = 0x50;
pub const REGISTRY_ERROR                                           : ULONG = 0x51;
pub const MAILSLOT_FILE_SYSTEM                                     : ULONG = 0x52;
pub const NO_BOOT_DEVICE                                           : ULONG = 0x53;
pub const LM_SERVER_INTERNAL_ERROR                                 : ULONG = 0x54;
pub const DATA_COHERENCY_EXCEPTION                                 : ULONG = 0x55;
pub const INSTRUCTION_COHERENCY_EXCEPTION                          : ULONG = 0x56;
pub const XNS_INTERNAL_ERROR                                       : ULONG = 0x57;
pub const FTDISK_INTERNAL_ERROR                                    : ULONG = 0x58;
pub const PINBALL_FILE_SYSTEM                                      : ULONG = 0x59;
pub const CRITICAL_SERVICE_FAILED                                  : ULONG = 0x5A;
pub const SET_ENV_VAR_FAILED                                       : ULONG = 0x5B;
pub const HAL_INITIALIZATION_FAILED                                : ULONG = 0x5C;
pub const UNSUPPORTED_PROCESSOR                                    : ULONG = 0x5D;
pub const OBJECT_INITIALIZATION_FAILED                             : ULONG = 0x5E;
pub const SECURITY_INITIALIZATION_FAILED                           : ULONG = 0x5F;
pub const PROCESS_INITIALIZATION_FAILED                            : ULONG = 0x60;
pub const HAL1_INITIALIZATION_FAILED                               : ULONG = 0x61;
pub const OBJECT1_INITIALIZATION_FAILED                            : ULONG = 0x62;
pub const SECURITY1_INITIALIZATION_FAILED                          : ULONG = 0x63;
pub const SYMBOLIC_INITIALIZATION_FAILED                           : ULONG = 0x64;
pub const MEMORY1_INITIALIZATION_FAILED                            : ULONG = 0x65;
pub const CACHE_INITIALIZATION_FAILED                              : ULONG = 0x66;
pub const CONFIG_INITIALIZATION_FAILED                             : ULONG = 0x67;
pub const FILE_INITIALIZATION_FAILED                               : ULONG = 0x68;
pub const IO1_INITIALIZATION_FAILED                                : ULONG = 0x69;
pub const LPC_INITIALIZATION_FAILED                                : ULONG = 0x6A;
pub const PROCESS1_INITIALIZATION_FAILED                           : ULONG = 0x6B;
pub const REFMON_INITIALIZATION_FAILED                             : ULONG = 0x6C;
pub const SESSION1_INITIALIZATION_FAILED                           : ULONG = 0x6D;
pub const SESSION2_INITIALIZATION_FAILED                           : ULONG = 0x6E;
pub const SESSION3_INITIALIZATION_FAILED                           : ULONG = 0x6F;
pub const SESSION4_INITIALIZATION_FAILED                           : ULONG = 0x70;
pub const SESSION5_INITIALIZATION_FAILED                           : ULONG = 0x71;
pub const ASSIGN_DRIVE_LETTERS_FAILED                              : ULONG = 0x72;
pub const CONFIG_LIST_FAILED                                       : ULONG = 0x73;
pub const BAD_SYSTEM_CONFIG_INFO                                   : ULONG = 0x74;
pub const CANNOT_WRITE_CONFIGURATION                               : ULONG = 0x75;
pub const PROCESS_HAS_LOCKED_PAGES                                 : ULONG = 0x76;
pub const KERNEL_STACK_INPAGE_ERROR                                : ULONG = 0x77;
pub const PHASE0_EXCEPTION                                         : ULONG = 0x78;
pub const MISMATCHED_HAL                                           : ULONG = 0x79;
pub const KERNEL_DATA_INPAGE_ERROR                                 : ULONG = 0x7A;
pub const INACCESSIBLE_BOOT_DEVICE                                 : ULONG = 0x7B;
pub const BUGCODE_NDIS_DRIVER                                      : ULONG = 0x7C;
pub const INSTALL_MORE_MEMORY                                      : ULONG = 0x7D;
pub const SYSTEM_THREAD_EXCEPTION_NOT_HANDLED                      : ULONG = 0x7E;
pub const UNEXPECTED_KERNEL_MODE_TRAP                              : ULONG = 0x7F;
pub const NMI_HARDWARE_FAILURE                                     : ULONG = 0x80;
pub const SPIN_LOCK_INIT_FAILURE                                   : ULONG = 0x81;
pub const KERNEL_MODE_EXCEPTION_NOT_HANDLED                        : ULONG = 0x8E;
pub const PP0_INITIALIZATION_FAILED                                : ULONG = 0x8F;
pub const PP1_INITIALIZATION_FAILED                                : ULONG = 0x90;
pub const WIN32K_INIT_OR_RIT_FAILURE                               : ULONG = 0x91;
pub const UP_DRIVER_ON_MP_SYSTEM                                   : ULONG = 0x92;
pub const INVALID_KERNEL_HANDLE                                    : ULONG = 0x93;
pub const KERNEL_STACK_LOCKED_AT_EXIT                              : ULONG = 0x94;
pub const INVALID_WORK_QUEUE_ITEM                                  : ULONG = 0x96;
pub const MORAL_EXCEPTION_ERROR                                    : ULONG = 0x9A;
pub const DRIVER_POWER_STATE_FAILURE                               : ULONG = 0x9F;
pub const INTERNAL_POWER_ERROR                                     : ULONG = 0xA0;
pub const PCI_BUS_DRIVER_INTERNAL                                  : ULONG = 0xA1;
pub const ACPI_BIOS_ERROR                                          : ULONG = 0xA5;
pub const HAL_MEMORY_ALLOCATION                                    : ULONG = 0xAC;
pub const VIDEO_DRIVER_INIT_FAILURE                                : ULONG = 0xB4;
pub const ATTEMPTED_SWITCH_FROM_DPC                                : ULONG = 0xB8;
pub const ATTEMPTED_WRITE_TO_READONLY_MEMORY                       : ULONG = 0xBE;
pub const SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION                  : ULONG = 0xC1;
pub const BAD_POOL_CALLER                                          : ULONG = 0xC2;
pub const BUGCODE_PSS_MESSAGE_SIGNATURE                            : ULONG = 0xC3;
pub const DRIVER_VERIFIER_DETECTED_VIOLATION                       : ULONG = 0xC4;
pub const DRIVER_CORRUPTED_EXPOOL                                  : ULONG = 0xC5;
pub const DRIVER_CAUGHT_MODIFYING_FREED_POOL                       : ULONG = 0xC6;
pub const IRQL_UNEXPECTED_VALUE                                    : ULONG = 0xC8;
pub const PNP_DETECTED_FATAL_ERROR                                 : ULONG = 0xCA;
pub const DRIVER_LEFT_LOCKED_PAGES_IN_PROCESS                      : ULONG = 0xCB;
pub const PAGE_FAULT_IN_FREED_SPECIAL_POOL                         : ULONG = 0xCC;
pub const PAGE_FAULT_BEYOND_END_OF_ALLOCATION                      : ULONG = 0xCD;
pub const DRIVER_UNLOADED_WITHOUT_CANCELLING_PENDING_OPERATIONS    : ULONG = 0xCE;
pub const DRIVER_CORRUPTED_MMPOOL                                  : ULONG = 0xD0;
pub const DRIVER_IRQL_NOT_LESS_OR_EQUAL                            : ULONG = 0xD1;
pub const DRIVER_PORTION_MUST_BE_NONPAGED                          : ULONG = 0xD3;
pub const SYSTEM_SCAN_AT_RAISED_IRQL_CAUGHT_IMPROPER_DRIVER_UNLOAD : ULONG = 0xD4;
pub const DRIVER_PAGE_FAULT_IN_FREED_SPECIAL_POOL                  : ULONG = 0xD5;
pub const DRIVER_PAGE_FAULT_BEYOND_END_OF_ALLOCATION               : ULONG = 0xD6;
pub const DRIVER_UNMAPPING_INVALID_VIEW                            : ULONG = 0xD7;
pub const DRIVER_USED_EXCESSIVE_PTES                               : ULONG = 0xD8;
pub const LOCKED_PAGES_TRACKER_CORRUPTION                          : ULONG = 0xD9;
pub const SYSTEM_PTE_MISUSE                                        : ULONG = 0xDA;
pub const DRIVER_CORRUPTED_SYSPTES                                 : ULONG = 0xDB;
pub const DRIVER_INVALID_STACK_ACCESS                              : ULONG = 0xDC;
pub const POOL_CORRUPTION_IN_FILE_AREA                             : ULONG = 0xDE;
pub const IMPERSONATING_WORKER_THREAD                              : ULONG = 0xDF;
pub const ACPI_BIOS_FATAL_ERROR                                    : ULONG = 0xE0;
pub const WORKER_THREAD_RETURNED_AT_BAD_IRQL                       : ULONG = 0xE1;
pub const MANUALLY_INITIATED_CRASH                                 : ULONG = 0xE2;
pub const RESOURCE_NOT_OWNED                                       : ULONG = 0xE3;
pub const WORKER_INVALID                                           : ULONG = 0xE4;
pub const POWER_FAILURE_SIMULATE                                   : ULONG = 0xE5;
pub const INVALID_FLOATING_POINT_STATE                             : ULONG = 0xE7;
pub const INVALID_CANCEL_OF_FILE_OPEN                              : ULONG = 0xE8;
pub const ACTIVE_EX_WORKER_THREAD_TERMINATION                      : ULONG = 0xE9;
pub const THREAD_STUCK_IN_DEVICE_DRIVER                            : ULONG = 0xEA;
pub const CRITICAL_PROCESS_DIED                                    : ULONG = 0xEF;
pub const CRITICAL_OBJECT_TERMINATION                              : ULONG = 0xF4;
pub const PCI_VERIFIER_DETECTED_VIOLATION                          : ULONG = 0xF6;
pub const DRIVER_OVERRAN_STACK_BUFFER                              : ULONG = 0xF7;
pub const RAMDISK_BOOT_INITIALIZATION_FAILED                       : ULONG = 0xF8;
pub const DRIVER_RETURNED_STATUS_REPARSE_FOR_VOLUME_OPEN           : ULONG = 0xF9;
pub const HTTP_DRIVER_CORRUPTED                                    : ULONG = 0xFA;
pub const ATTEMPTED_EXECUTE_OF_NOEXECUTE_MEMORY                    : ULONG = 0xFC;
pub const DIRTY_NOWRITE_PAGES_CONGESTION                           : ULONG = 0xFD;
pub const BUGCODE_USB_DRIVER                                       : ULONG = 0xFE;
pub const KERNEL_SECURITY_CHECK_FAILURE                            : ULONG = 0x139;
pub const WINDOWS_NT_BANNER                                        : ULONG = 0x4000007E;  // message string, NOT a stop code
pub const WINDOWS_NT_CSD_STRING                                    : ULONG = 0x40000087;  // message string, NOT a stop code
pub const WINDOWS_NT_INFO_STRING                                   : ULONG = 0x40000088;  // message string, NOT a stop code
pub const WINDOWS_NT_MP_STRING                                     : ULONG = 0x40000089;  // message string, NOT a stop code
pub const THREAD_TERMINATE_HELD_MUTEX                              : ULONG = 0x4000008A;  // message string, NOT a stop code
pub const WINDOWS_NT_INFO_STRING_PLURAL                            : ULONG = 0x4000009D;  // message string, NOT a stop code
pub const REACTOS_COPYRIGHT_NOTICE                                 : ULONG = 0x4000009F;  // message string, NOT a stop code
pub const BOOTING_IN_SAFEMODE_MINIMAL                              : ULONG = 0x400000A8;  // message string, NOT a stop code
pub const BOOTING_IN_SAFEMODE_NETWORK                              : ULONG = 0x400000A9;  // message string, NOT a stop code
pub const BOOTING_IN_SAFEMODE_DSREPAIR                             : ULONG = 0x400000AA;  // message string, NOT a stop code
pub const BOOTLOG_ENABLED                                          : ULONG = 0x400000B7;  // message string, NOT a stop code
pub const BUGCHECK_MESSAGE_INTRO                                   : ULONG = 0x8000007F;  // message string, NOT a stop code
pub const BUGCODE_ID_DRIVER                                        : ULONG = 0x80000080;  // message string, NOT a stop code
pub const PSS_MESSAGE_INTRO                                        : ULONG = 0x80000081;  // message string, NOT a stop code
pub const BUGCODE_PSS_MESSAGE                                      : ULONG = 0x80000082;  // message string, NOT a stop code
pub const BUGCHECK_TECH_INFO                                       : ULONG = 0x80000083;  // message string, NOT a stop code

/// Real-NT stop code that ReactOS's bugcodes.mc does not define. Used by our own
/// IPI/spinlock stall guards, which have no C original to inherit a code from.
/// Kept here so those call sites stay symbolic like every other one.
pub const CLOCK_WATCHDOG_TIMEOUT: ULONG = 0x101;
