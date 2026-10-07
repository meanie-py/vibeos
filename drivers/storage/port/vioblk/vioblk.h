/*
 * vioblk.sys — virtio-blk SCSIPORT miniport. The driver is Rust
 * (sdk/lib/rust/vioblk-rust); this header exists only so the bindgen substrate
 * (tools/rustconv/gen_driver_types.sh vioblk) emits the exact SCSIPORT miniport
 * layouts (HW_INITIALIZATION_DATA, PORT_CONFIGURATION_INFORMATION, SRB, CDB,
 * INQUIRYDATA, READ_CAPACITY_DATA) for all three arches.
 */
#pragma once
#include <ntddk.h>
#include <srb.h>
#include <scsi.h>
