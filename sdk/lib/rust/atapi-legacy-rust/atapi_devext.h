/*
 * Bindgen-only header for atapi-rust's gen_driver_types.sh substrate.
 *
 * HW_DEVICE_EXTENSION and HW_LU_EXTENSION are defined inside atapi.c (not a
 * header), so they are reproduced here VERBATIM from
 * drivers/storage/ide/atapi/atapi.c so bindgen can emit their exact per-arch
 * layouts. This file is NOT compiled into the C driver; it is consumed only by
 * tools/rustconv/gen_driver_types.sh (the wrapper #includes atapi.h first, so
 * IDENTIFY_DATA / SCSI_REQUEST_BLOCK / SENSE_DATA / MECHANICAL_STATUS_* are in
 * scope). Keep in sync with atapi.c if that struct ever changes.
 */

typedef struct _HW_DEVICE_EXTENSION {
    PSCSI_REQUEST_BLOCK CurrentSrb;
    PIDE_REGISTERS_1 BaseIoAddress1[2];
    PIDE_REGISTERS_2 BaseIoAddress2[2];
    ULONG InterruptLevel;
    ULONG InterruptMode;
    PUSHORT DataBuffer;
    ULONG WordsLeft;
    ULONG NumberChannels;
    ULONG ErrorCount;
    ULONG DiscsPresent[4];
    USHORT DeviceFlags[4];
    UCHAR MaximumBlockXfer[4];
    BOOLEAN ExpectingInterrupt;
    BOOLEAN RDP;
    BOOLEAN DriverMustPoll;
    BOOLEAN DWordIO;
    BOOLEAN PrimaryAddress;
    UCHAR SmartCommand;
    UCHAR ReturningMediaStatus;
    UCHAR Reserved[1];
    IDENTIFY_DATA FullIdentifyData;
    IDENTIFY_DATA2 IdentifyData[4];
    PSCSI_REQUEST_BLOCK OriginalSrb;
    SCSI_REQUEST_BLOCK InternalSrb;
    MECHANICAL_STATUS_INFORMATION_HEADER MechStatusData;
    SENSE_DATA MechStatusSense;
    ULONG MechStatusRetryCount;
} HW_DEVICE_EXTENSION, *PHW_DEVICE_EXTENSION;

typedef struct _HW_LU_EXTENSION {
   ULONG Reserved;
} HW_LU_EXTENSION, *PHW_LU_EXTENSION;
