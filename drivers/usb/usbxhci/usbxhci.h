/*
 * PROJECT:     VibeOS USB xHCI Miniport Driver
 * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
 * PURPOSE:     Header for the bindgen type island of usbxhci-rust
 *
 * The driver itself is Rust (sdk/lib/rust/usbxhci-rust), written fresh against
 * the xHCI 1.2 specification; see docs/engineering/usbxhci-design.md. This
 * header exists only so tools/rustconv/gen_driver_types.sh can emit the exact
 * usbport miniport interface (usbmport.h) and USB descriptor layouts for all
 * three arches. xHCI hardware structures (TRBs, contexts, registers) are not in
 * any SDK header; they are defined in Rust with size assertions (hw.rs).
 */

#ifndef USBXHCI_H__
#define USBXHCI_H__

#include <ntddk.h>
#include <windef.h>
#include <hubbusif.h>
#include <usbbusif.h>
#include <usbdlib.h>
#include <drivers/usbport/usbmport.h>

#endif /* USBXHCI_H__ */
