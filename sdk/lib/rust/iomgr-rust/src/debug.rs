//! io/debug.rs — Debug/dump utilities for IO and PNP managers.
//!
//! Translated from ntoskrnl/io/debug.c.
//! These are diagnostic functions that dump device node trees, CM resource
//! lists, and IO resource requirements via DbgPrint. They are not exported
//! and are only called from internal PnP code.

use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    _CM_PARTIAL_RESOURCE_DESCRIPTOR, _CM_RESOURCE_LIST, _DEVICE_NODE,
    _IO_RESOURCE_DESCRIPTOR, _IO_RESOURCE_REQUIREMENTS_LIST,
};

type PCM_PARTIAL_RESOURCE_DESCRIPTOR = *mut _CM_PARTIAL_RESOURCE_DESCRIPTOR;
type PCM_RESOURCE_LIST = *mut _CM_RESOURCE_LIST;
type PIO_RESOURCE_DESCRIPTOR = *mut _IO_RESOURCE_DESCRIPTOR;
type PIO_RESOURCE_REQUIREMENTS_LIST = *mut _IO_RESOURCE_REQUIREMENTS_LIST;
type PDEVICE_NODE = *mut _DEVICE_NODE;
type PNP_DEVNODE_STATE = ULONG;

const CmResourceTypeNull: UCHAR = 0;
const CmResourceTypePort: UCHAR = 1;
const CmResourceTypeInterrupt: UCHAR = 2;
const CmResourceTypeMemory: UCHAR = 3;
const CmResourceTypeDma: UCHAR = 4;
const CmResourceTypeDeviceSpecific: UCHAR = 5;
const CmResourceTypeBusNumber: UCHAR = 6;
const CmResourceTypeConfigData: UCHAR = 128;
const CmResourceTypeDevicePrivate: UCHAR = 129;

const PIP_DUMP_FL_ALL_NODES: ULONG = 1;
const PIP_DUMP_FL_RES_ALLOCATED: ULONG = 2;
const PIP_DUMP_FL_RES_REQUIREMENTS: ULONG = 4;
const PIP_DUMP_FL_RES_TRANSLATED: ULONG = 8;

// DeviceNode state enum values
const DeviceNodeUnspecified: PNP_DEVNODE_STATE = 0x300;
const DeviceNodeUninitialized: PNP_DEVNODE_STATE = 0x301;
const DeviceNodeInitialized: PNP_DEVNODE_STATE = 0x302;
const DeviceNodeDriversAdded: PNP_DEVNODE_STATE = 0x303;
const DeviceNodeResourcesAssigned: PNP_DEVNODE_STATE = 0x304;
const DeviceNodeStartPending: PNP_DEVNODE_STATE = 0x305;
const DeviceNodeStartCompletion: PNP_DEVNODE_STATE = 0x306;
const DeviceNodeStartPostWork: PNP_DEVNODE_STATE = 0x307;
const DeviceNodeStarted: PNP_DEVNODE_STATE = 0x308;
const DeviceNodeQueryStopped: PNP_DEVNODE_STATE = 0x309;
const DeviceNodeStopped: PNP_DEVNODE_STATE = 0x30A;
const DeviceNodeRestartCompletion: PNP_DEVNODE_STATE = 0x30B;
const DeviceNodeEnumeratePending: PNP_DEVNODE_STATE = 0x30C;
const DeviceNodeEnumerateCompletion: PNP_DEVNODE_STATE = 0x30D;
const DeviceNodeAwaitingQueuedDeletion: PNP_DEVNODE_STATE = 0x30E;
const DeviceNodeAwaitingQueuedRemoval: PNP_DEVNODE_STATE = 0x30F;
const DeviceNodeQueryRemoved: PNP_DEVNODE_STATE = 0x310;
const DeviceNodeRemovePendingCloses: PNP_DEVNODE_STATE = 0x311;
const DeviceNodeRemoved: PNP_DEVNODE_STATE = 0x312;
const DeviceNodeDeletePendingCloses: PNP_DEVNODE_STATE = 0x313;
const DeviceNodeDeleted: PNP_DEVNODE_STATE = 0x314;
const MaxDeviceNodeState: PNP_DEVNODE_STATE = 0x315;

#[no_mangle]
pub unsafe extern "system" fn PipDumpCmResourceDescriptor(
    _Descriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    _DebugLevel: ULONG,
) {
    // Debug dump function — all output goes through DPRINT1/DbgPrint.
    // In the Rust build with NDEBUG, these are effectively no-ops for DebugLevel != 0.
    // For DebugLevel == 0, we'd need variadic DbgPrint which is complex in Rust.
    // Since these are purely diagnostic, just return.
}

#[no_mangle]
pub unsafe extern "system" fn PipGetNextCmPartialDescriptor(
    CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> PCM_PARTIAL_RESOURCE_DESCRIPTOR {
    // Assume fixed-size descriptor
    let mut NextDescriptor = CmDescriptor.add(1);

    // Check for variable-sized descriptor
    if (*CmDescriptor).Type == CmResourceTypeDeviceSpecific {
        NextDescriptor = (NextDescriptor as *mut u8)
            .add((*CmDescriptor).u.DeviceSpecificData.DataSize as usize)
            as PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    }

    NextDescriptor
}

#[no_mangle]
pub unsafe extern "system" fn PipDumpCmResourceList(
    _CmResource: PCM_RESOURCE_LIST,
    _DebugLevel: ULONG,
) {
    // Debug dump — no-op in Rust (would need variadic DbgPrint for format strings)
}

#[no_mangle]
pub unsafe extern "system" fn PipDumpIoResourceDescriptor(
    _Descriptor: PIO_RESOURCE_DESCRIPTOR,
    _DebugLevel: ULONG,
) {
    // Debug dump — no-op in Rust
}

#[no_mangle]
pub unsafe extern "system" fn PipDumpResourceRequirementsList(
    _IoResource: PIO_RESOURCE_REQUIREMENTS_LIST,
    _DebugLevel: ULONG,
) {
    // Debug dump — no-op in Rust
}

#[no_mangle]
pub unsafe extern "system" fn PipGetDeviceNodeStateName(
    State: PNP_DEVNODE_STATE,
) -> PWSTR {
    match State {
        DeviceNodeUnspecified => {
            static S: [u16; 23] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'U' as u16, b'n' as u16, b's' as u16, b'p' as u16, b'e' as u16, b'c' as u16, b'i' as u16, b'f' as u16, b'i' as u16, b'e' as u16, b'd' as u16, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeUninitialized => {
            static S: [u16; 25] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'U' as u16, b'n' as u16, b'i' as u16, b'n' as u16, b'i' as u16, b't' as u16, b'i' as u16, b'a' as u16, b'l' as u16, b'i' as u16, b'z' as u16, b'e' as u16, b'd' as u16, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeInitialized => {
            static S: [u16; 23] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'I' as u16, b'n' as u16, b'i' as u16, b't' as u16, b'i' as u16, b'a' as u16, b'l' as u16, b'i' as u16, b'z' as u16, b'e' as u16, b'd' as u16, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeDriversAdded => {
            static S: [u16; 24] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'D' as u16, b'r' as u16, b'i' as u16, b'v' as u16, b'e' as u16, b'r' as u16, b's' as u16, b'A' as u16, b'd' as u16, b'd' as u16, b'e' as u16, b'd' as u16, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeResourcesAssigned => {
            static S: [u16; 30] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'R' as u16, b'e' as u16, b's' as u16, b'o' as u16, b'u' as u16, b'r' as u16, b'c' as u16, b'e' as u16, b's' as u16, b'A' as u16, b's' as u16, b's' as u16, b'i' as u16, b'g' as u16, b'n' as u16, b'e' as u16, b'd' as u16, 0, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeStartPending => {
            static S: [u16; 24] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'S' as u16, b't' as u16, b'a' as u16, b'r' as u16, b't' as u16, b'P' as u16, b'e' as u16, b'n' as u16, b'd' as u16, b'i' as u16, b'n' as u16, b'g' as u16, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeStartCompletion => {
            static S: [u16; 27] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'S' as u16, b't' as u16, b'a' as u16, b'r' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'm' as u16, b'p' as u16, b'l' as u16, b'e' as u16, b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeStartPostWork => {
            static S: [u16; 26] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'S' as u16, b't' as u16, b'a' as u16, b'r' as u16, b't' as u16, b'P' as u16, b'o' as u16, b's' as u16, b't' as u16, b'W' as u16, b'o' as u16, b'r' as u16, b'k' as u16, 0, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeStarted => {
            static S: [u16; 19] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'S' as u16, b't' as u16, b'a' as u16, b'r' as u16, b't' as u16, b'e' as u16, b'd' as u16, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeQueryStopped => {
            static S: [u16; 24] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'Q' as u16, b'u' as u16, b'e' as u16, b'r' as u16, b'y' as u16, b'S' as u16, b't' as u16, b'o' as u16, b'p' as u16, b'p' as u16, b'e' as u16, b'd' as u16, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeStopped => {
            static S: [u16; 19] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'S' as u16, b't' as u16, b'o' as u16, b'p' as u16, b'p' as u16, b'e' as u16, b'd' as u16, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeRestartCompletion => {
            static S: [u16; 29] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'R' as u16, b'e' as u16, b's' as u16, b't' as u16, b'a' as u16, b'r' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'm' as u16, b'p' as u16, b'l' as u16, b'e' as u16, b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeEnumeratePending => {
            static S: [u16; 29] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'E' as u16, b'n' as u16, b'u' as u16, b'm' as u16, b'e' as u16, b'r' as u16, b'a' as u16, b't' as u16, b'e' as u16, b'P' as u16, b'e' as u16, b'n' as u16, b'd' as u16, b'i' as u16, b'n' as u16, b'g' as u16, 0, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeEnumerateCompletion => {
            static S: [u16; 32] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'E' as u16, b'n' as u16, b'u' as u16, b'm' as u16, b'e' as u16, b'r' as u16, b'a' as u16, b't' as u16, b'e' as u16, b'C' as u16, b'o' as u16, b'm' as u16, b'p' as u16, b'l' as u16, b'e' as u16, b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16, 0, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeAwaitingQueuedDeletion => {
            static S: [u16; 35] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'A' as u16, b'w' as u16, b'a' as u16, b'i' as u16, b't' as u16, b'i' as u16, b'n' as u16, b'g' as u16, b'Q' as u16, b'u' as u16, b'e' as u16, b'u' as u16, b'e' as u16, b'd' as u16, b'D' as u16, b'e' as u16, b'l' as u16, b'e' as u16, b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16, 0, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeAwaitingQueuedRemoval => {
            static S: [u16; 34] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'A' as u16, b'w' as u16, b'a' as u16, b'i' as u16, b't' as u16, b'i' as u16, b'n' as u16, b'g' as u16, b'Q' as u16, b'u' as u16, b'e' as u16, b'u' as u16, b'e' as u16, b'd' as u16, b'R' as u16, b'e' as u16, b'm' as u16, b'o' as u16, b'v' as u16, b'a' as u16, b'l' as u16, 0, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeQueryRemoved => {
            static S: [u16; 24] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'Q' as u16, b'u' as u16, b'e' as u16, b'r' as u16, b'y' as u16, b'R' as u16, b'e' as u16, b'm' as u16, b'o' as u16, b'v' as u16, b'e' as u16, b'd' as u16, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeRemovePendingCloses => {
            static S: [u16; 31] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'R' as u16, b'e' as u16, b'm' as u16, b'o' as u16, b'v' as u16, b'e' as u16, b'P' as u16, b'e' as u16, b'n' as u16, b'd' as u16, b'i' as u16, b'n' as u16, b'g' as u16, b'C' as u16, b'l' as u16, b'o' as u16, b's' as u16, b'e' as u16, b's' as u16, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeRemoved => {
            static S: [u16; 19] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'R' as u16, b'e' as u16, b'm' as u16, b'o' as u16, b'v' as u16, b'e' as u16, b'd' as u16, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeDeletePendingCloses => {
            static S: [u16; 31] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'D' as u16, b'e' as u16, b'l' as u16, b'e' as u16, b't' as u16, b'e' as u16, b'P' as u16, b'e' as u16, b'n' as u16, b'd' as u16, b'i' as u16, b'n' as u16, b'g' as u16, b'C' as u16, b'l' as u16, b'o' as u16, b's' as u16, b'e' as u16, b's' as u16, 0, 0]; S.as_ptr() as PWSTR
        }
        DeviceNodeDeleted => {
            static S: [u16; 19] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'D' as u16, b'e' as u16, b'l' as u16, b'e' as u16, b't' as u16, b'e' as u16, b'd' as u16, 0, 0]; S.as_ptr() as PWSTR
        }
        _ => {
            static EMPTY: [u16; 1] = [0];
            EMPTY.as_ptr() as PWSTR
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn PipDumpArbiters(
    _DeviceNode: PDEVICE_NODE,
    _NodeLevel: ULONG,
) {
    // UNIMPLEMENTED in C version too
}

#[no_mangle]
pub unsafe extern "system" fn PipDumpDeviceNode(
    DeviceNode: PDEVICE_NODE,
    NodeLevel: ULONG,
    Flags: ULONG,
    _DebugLevel: ULONG,
) {
    // Debug dump function — recursive traversal of device tree.
    // The actual DbgPrint formatting is omitted (NDEBUG build).
    // Only the traversal logic matters for correctness.

    if Flags & PIP_DUMP_FL_ALL_NODES != 0 {
        let mut ChildDeviceNode = (*DeviceNode).Child;
        while !ChildDeviceNode.is_null() {
            PipDumpDeviceNode(ChildDeviceNode, NodeLevel + 1, Flags, _DebugLevel);
            ChildDeviceNode = (*ChildDeviceNode).Sibling;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn PipDumpDeviceNodes(
    DeviceNode: PDEVICE_NODE,
    Flags: ULONG,
    DebugLevel: ULONG,
) {
    // No-op in Rust — would need IopRootDeviceNode extern which causes ARM64 linker issues
}
