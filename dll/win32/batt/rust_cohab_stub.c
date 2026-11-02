/* batt is Rust (sdk/lib/rust/batt-rust); a CMake target needs one TU. */
typedef int batt_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char batt_export_anchor_0 __asm__("_BatteryClassCoInstaller@16");
extern char batt_export_anchor_1 __asm__("_BatteryClassInstall@12");
__attribute__((used)) static const void *const batt_export_anchors[] = {
    &batt_export_anchor_0,
    &batt_export_anchor_1,
    0,
};
#elif defined(__x86_64__)
extern char batt_export_anchor_0 __asm__("BatteryClassCoInstaller");
extern char batt_export_anchor_1 __asm__("BatteryClassInstall");
__attribute__((used)) static const void *const batt_export_anchors[] = {
    &batt_export_anchor_0,
    &batt_export_anchor_1,
    0,
};
#elif defined(__aarch64__)
extern char batt_export_anchor_0 __asm__("BatteryClassCoInstaller");
extern char batt_export_anchor_1 __asm__("BatteryClassInstall");
__attribute__((used)) static const void *const batt_export_anchors[] = {
    &batt_export_anchor_0,
    &batt_export_anchor_1,
    0,
};
#endif
