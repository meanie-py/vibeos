//! Bare enum-variant aliases: bindgen prefixes constified enumerators with their
//! enum name, while the translated bodies spell them as the C does. Kept here, not in
//! the generated types.rs, so regenerating the island does not drop them.
#![allow(non_upper_case_globals)]
pub use super::types_arch::_PCI_DEVICE_TYPES_PciTypeCardbusBridge as PciTypeCardbusBridge;
pub use super::types_arch::_PCI_DEVICE_TYPES_PciTypeDevice as PciTypeDevice;
pub use super::types_arch::_PCI_DEVICE_TYPES_PciTypeHostBridge as PciTypeHostBridge;
pub use super::types_arch::_PCI_DEVICE_TYPES_PciTypeInvalid as PciTypeInvalid;
pub use super::types_arch::_PCI_DEVICE_TYPES_PciTypePciBridge as PciTypePciBridge;
pub use super::types_arch::_PCI_DISPATCH_STYLE_IRP_COMPLETE as IRP_COMPLETE;
pub use super::types_arch::_PCI_DISPATCH_STYLE_IRP_DISPATCH as IRP_DISPATCH;
pub use super::types_arch::_PCI_DISPATCH_STYLE_IRP_DOWNWARD as IRP_DOWNWARD;
pub use super::types_arch::_PCI_DISPATCH_STYLE_IRP_UPWARD as IRP_UPWARD;
pub use super::types_arch::_PCI_SIGNATURE_PciArb_BusNumber as PciArb_BusNumber;
pub use super::types_arch::_PCI_SIGNATURE_PciArb_Interrupt as PciArb_Interrupt;
pub use super::types_arch::_PCI_SIGNATURE_PciArb_Io as PciArb_Io;
pub use super::types_arch::_PCI_SIGNATURE_PciArb_Memory as PciArb_Memory;
pub use super::types_arch::_PCI_SIGNATURE_PciFdoExtensionType as PciFdoExtensionType;
pub use super::types_arch::_PCI_SIGNATURE_PciInterface_AgpTarget as PciInterface_AgpTarget;
pub use super::types_arch::_PCI_SIGNATURE_PciInterface_BusHandler as PciInterface_BusHandler;
pub use super::types_arch::_PCI_SIGNATURE_PciInterface_DevicePresent as PciInterface_DevicePresent;
pub use super::types_arch::_PCI_SIGNATURE_PciInterface_IntRouteHandler as PciInterface_IntRouteHandler;
pub use super::types_arch::_PCI_SIGNATURE_PciInterface_LegacyDeviceDetection as PciInterface_LegacyDeviceDetection;
pub use super::types_arch::_PCI_SIGNATURE_PciInterface_Location as PciInterface_Location;
pub use super::types_arch::_PCI_SIGNATURE_PciInterface_NativeIde as PciInterface_NativeIde;
pub use super::types_arch::_PCI_SIGNATURE_PciInterface_PciCb as PciInterface_PciCb;
pub use super::types_arch::_PCI_SIGNATURE_PciInterface_PmeHandler as PciInterface_PmeHandler;
pub use super::types_arch::_PCI_SIGNATURE_PciPdoExtensionType as PciPdoExtensionType;
pub use super::types_arch::_PCI_SIGNATURE_PciTrans_Interrupt as PciTrans_Interrupt;
pub use super::types_arch::_PCI_STATE_PciDeleted as PciDeleted;
pub use super::types_arch::_PCI_STATE_PciMaxObjectState as PciMaxObjectState;
pub use super::types_arch::_PCI_STATE_PciNotStarted as PciNotStarted;
pub use super::types_arch::_PCI_STATE_PciStarted as PciStarted;
pub use super::types_arch::_PCI_STATE_PciStopped as PciStopped;
pub use super::types_arch::_PCI_STATE_PciSurpriseRemoved as PciSurpriseRemoved;
pub use super::types_arch::_PCI_STATE_PciSynchronizedOperation as PciSynchronizedOperation;
