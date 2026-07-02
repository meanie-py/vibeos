//! buslogic-rust glue: bare-named enum-member aliases the c2rust/LLM translation references
//! (bindgen names them `<EnumTag>_<member>`; the driver source used the bare `<member>`),
//! plus NULL / INTERFACE_TYPE / SP_UNTAGGED not emitted as bare consts. `pub use ... as`
//! re-exports by the bindgen full-name → arch-independent (same enum names on all 3 arches).
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use crate::types::*;
use ntkrnl_sys::base::{PVOID, UCHAR};

pub type INTERFACE_TYPE = ntkrnl_sys::ddk::_INTERFACE_TYPE;
pub const NULL: PVOID = core::ptr::null_mut();
pub const SP_UNTAGGED: UCHAR = 0xFF; // scsi.h: untagged-request queue tag

// ---- auto-generated enum-member aliases (gen_driver_types.sh substrate) ----
pub use crate::types::BusLogic_CompletionCode_T_BusLogic_AbortedCommandNotFound as BusLogic_AbortedCommandNotFound;
pub use crate::types::BusLogic_CCB_Opcode_T_BusLogic_BusDeviceReset as BusLogic_BusDeviceReset;
pub use crate::types::BusLogic_CCB_Status_T_BusLogic_CCB_Active as BusLogic_CCB_Active;
pub use crate::types::BusLogic_CCB_Status_T_BusLogic_CCB_Completed as BusLogic_CCB_Completed;
pub use crate::types::BusLogic_CCB_Status_T_BusLogic_CCB_Free as BusLogic_CCB_Free;
pub use crate::types::BusLogic_CCB_Status_T_BusLogic_CCB_Reset as BusLogic_CCB_Reset;
pub use crate::types::BusLogic_HostAdapterStatus_T_BusLogic_CommandCompletedNormally as BusLogic_CommandCompletedNormally;
pub use crate::types::BusLogic_DataDirection_T_BusLogic_DataInLengthChecked as BusLogic_DataInLengthChecked;
pub use crate::types::BusLogic_DataDirection_T_BusLogic_DataOutLengthChecked as BusLogic_DataOutLengthChecked;
pub use crate::types::BusLogic_OperationCode_T_BusLogic_DisableHostAdapterInterrupt as BusLogic_DisableHostAdapterInterrupt;
pub use crate::types::BusLogic_HostAdapterBusType_T_BusLogic_EISA_Bus as BusLogic_EISA_Bus;
pub use crate::types::BusLogic_OperationCode_T_BusLogic_EnableStrictRoundRobinMode as BusLogic_EnableStrictRoundRobinMode;
pub use crate::types::BusLogic_SetCCBFormatRequest_BusLogic_ExtendedLUNFormatCCB as BusLogic_ExtendedLUNFormatCCB;
pub use crate::types::BusLogic_OperationCode_T_BusLogic_FetchHostAdapterLocalRAM as BusLogic_FetchHostAdapterLocalRAM;
pub use crate::types::BusLogic_HostAdapterBusType_T_BusLogic_ISA_Bus as BusLogic_ISA_Bus;
pub use crate::types::BusLogic_CompletionCode_T_BusLogic_IncomingMailboxFree as BusLogic_IncomingMailboxFree;
pub use crate::types::BusLogic_OperationCode_T_BusLogic_InitializeExtendedMailbox as BusLogic_InitializeExtendedMailbox;
pub use crate::types::BusLogic_CCB_Opcode_T_BusLogic_InitiatorCCB as BusLogic_InitiatorCCB;
pub use crate::types::BusLogic_CCB_Opcode_T_BusLogic_InitiatorCCB_ScatterGather as BusLogic_InitiatorCCB_ScatterGather;
pub use crate::types::BusLogic_OperationCode_T_BusLogic_InquireBoardID as BusLogic_InquireBoardID;
pub use crate::types::BusLogic_OperationCode_T_BusLogic_InquireConfiguration as BusLogic_InquireConfiguration;
pub use crate::types::BusLogic_OperationCode_T_BusLogic_InquireExtendedSetupInformation as BusLogic_InquireExtendedSetupInformation;
pub use crate::types::BusLogic_OperationCode_T_BusLogic_InquireFirmwareVersion3rdDigit as BusLogic_InquireFirmwareVersion3rdDigit;
pub use crate::types::BusLogic_OperationCode_T_BusLogic_InquireFirmwareVersionLetter as BusLogic_InquireFirmwareVersionLetter;
pub use crate::types::BusLogic_OperationCode_T_BusLogic_InquireHostAdapterModelNumber as BusLogic_InquireHostAdapterModelNumber;
pub use crate::types::BusLogic_OperationCode_T_BusLogic_InquirePCIHostAdapterInformation as BusLogic_InquirePCIHostAdapterInformation;
pub use crate::types::BusLogic_OperationCode_T_BusLogic_InquireSetupInformation as BusLogic_InquireSetupInformation;
pub use crate::types::BusLogic_OperationCode_T_BusLogic_InquireSynchronousPeriod as BusLogic_InquireSynchronousPeriod;
pub use crate::types::BusLogic_OperationCode_T_BusLogic_InquireTargetDevices as BusLogic_InquireTargetDevices;
pub use crate::types::BusLogic_HostAdapterStatus_T_BusLogic_LinkedCommandCompleted as BusLogic_LinkedCommandCompleted;
pub use crate::types::BusLogic_HostAdapterStatus_T_BusLogic_LinkedCommandCompletedWithFlag as BusLogic_LinkedCommandCompletedWithFlag;
pub use crate::types::BusLogic_HostAdapterBusType_T_BusLogic_MCA_Bus as BusLogic_MCA_Bus;
pub use crate::types::BusLogic_ActionCode_T_BusLogic_MailboxAbortCommand as BusLogic_MailboxAbortCommand;
pub use crate::types::BusLogic_ActionCode_T_BusLogic_MailboxStartCommand as BusLogic_MailboxStartCommand;
pub use crate::types::BusLogic_OperationCode_T_BusLogic_ModifyIOAddress as BusLogic_ModifyIOAddress;
pub use crate::types::BusLogic_DataDirection_T_BusLogic_NoDataTransfer as BusLogic_NoDataTransfer;
pub use crate::types::BusLogic_TargetDeviceStatus_T_BusLogic_OperationGood as BusLogic_OperationGood;
pub use crate::types::BusLogic_ActionCode_T_BusLogic_OutgoingMailboxFree as BusLogic_OutgoingMailboxFree;
pub use crate::types::BusLogic_HostAdapterBusType_T_BusLogic_PCI_Bus as BusLogic_PCI_Bus;
pub use crate::types::BusLogic_HostAdapterStatus_T_BusLogic_SCSISelectionTimeout as BusLogic_SCSISelectionTimeout;
pub use crate::types::BusLogic_OperationCode_T_BusLogic_SetCCBFormat as BusLogic_SetCCBFormat;
pub use crate::types::BusLogic_QueueTag_T_BusLogic_SimpleQueueTag as BusLogic_SimpleQueueTag;
pub use crate::types::BusLogic_DataDirection_T_BusLogic_UncheckedDataTransfer as BusLogic_UncheckedDataTransfer;
pub use crate::types::BusLogic_HostAdapterBusType_T_BusLogic_Unknown_Bus as BusLogic_Unknown_Bus;
pub use crate::types::BusLogic_HostAdapterBusType_T_BusLogic_VESA_Bus as BusLogic_VESA_Bus;
pub use crate::types::_SCSI_NOTIFICATION_TYPE_NextLuRequest as NextLuRequest;
pub use crate::types::_SCSI_NOTIFICATION_TYPE_NextRequest as NextRequest;
pub use crate::types::_SCSI_NOTIFICATION_TYPE_RequestComplete as RequestComplete;
pub use crate::types::_SCSI_ADAPTER_CONTROL_TYPE_ScsiAdapterControlMax as ScsiAdapterControlMax;
pub use crate::types::_SCSI_ADAPTER_CONTROL_STATUS_ScsiAdapterControlSuccess as ScsiAdapterControlSuccess;
pub use crate::types::_SCSI_ADAPTER_CONTROL_STATUS_ScsiAdapterControlUnsuccessful as ScsiAdapterControlUnsuccessful;

// bare enum-member aliases for KINTERRUPT_MODE / INTERFACE_TYPE used in HW_INITIALIZATION_DATA / config
pub use ntkrnl_sys::ddk::_KINTERRUPT_MODE_LevelSensitive as LevelSensitive;
pub use ntkrnl_sys::ddk::_INTERFACE_TYPE_PCIBus as PCIBus;
