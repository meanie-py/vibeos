/* streamci is Rust (sdk/lib/rust/streamci-rust); a CMake target needs one TU. */
typedef int streamci_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char streamci_export_anchor_0 __asm__("_StreamingDeviceSetupW@16");
__attribute__((used)) static const void *const streamci_export_anchors[] = {
    &streamci_export_anchor_0,
    0,
};
#elif defined(__x86_64__)
extern char streamci_export_anchor_0 __asm__("StreamingDeviceSetupW");
__attribute__((used)) static const void *const streamci_export_anchors[] = {
    &streamci_export_anchor_0,
    0,
};
#elif defined(__aarch64__)
extern char streamci_export_anchor_0 __asm__("StreamingDeviceSetupW");
__attribute__((used)) static const void *const streamci_export_anchors[] = {
    &streamci_export_anchor_0,
    0,
};
#endif
