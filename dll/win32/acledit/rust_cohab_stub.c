/* acledit is Rust (sdk/lib/rust/acledit-rust); a CMake target needs one TU. */
typedef int acledit_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char acledit_export_anchor_0 __asm__("_DllMain@12");
extern char acledit_export_anchor_1 __asm__("_EditAuditInfo@4");
extern char acledit_export_anchor_2 __asm__("_EditOwnerInfo@4");
extern char acledit_export_anchor_3 __asm__("_EditPermissionInfo@4");
extern char acledit_export_anchor_4 __asm__("_FMExtensionProcW@12");
extern char acledit_export_anchor_5 __asm__("_SedDiscretionaryAclEditor@52");
extern char acledit_export_anchor_6 __asm__("_SedSystemAclEditor@48");
extern char acledit_export_anchor_7 __asm__("_SedTakeOwnership@56");
__attribute__((used)) static const void *const acledit_export_anchors[] = {
    &acledit_export_anchor_0,
    &acledit_export_anchor_1,
    &acledit_export_anchor_2,
    &acledit_export_anchor_3,
    &acledit_export_anchor_4,
    &acledit_export_anchor_5,
    &acledit_export_anchor_6,
    &acledit_export_anchor_7,
    0,
};
#elif defined(__x86_64__)
extern char acledit_export_anchor_0 __asm__("DllMain");
extern char acledit_export_anchor_1 __asm__("EditAuditInfo");
extern char acledit_export_anchor_2 __asm__("EditOwnerInfo");
extern char acledit_export_anchor_3 __asm__("EditPermissionInfo");
extern char acledit_export_anchor_4 __asm__("FMExtensionProcW");
extern char acledit_export_anchor_5 __asm__("SedDiscretionaryAclEditor");
extern char acledit_export_anchor_6 __asm__("SedSystemAclEditor");
extern char acledit_export_anchor_7 __asm__("SedTakeOwnership");
__attribute__((used)) static const void *const acledit_export_anchors[] = {
    &acledit_export_anchor_0,
    &acledit_export_anchor_1,
    &acledit_export_anchor_2,
    &acledit_export_anchor_3,
    &acledit_export_anchor_4,
    &acledit_export_anchor_5,
    &acledit_export_anchor_6,
    &acledit_export_anchor_7,
    0,
};
#elif defined(__aarch64__)
extern char acledit_export_anchor_0 __asm__("DllMain");
extern char acledit_export_anchor_1 __asm__("EditAuditInfo");
extern char acledit_export_anchor_2 __asm__("EditOwnerInfo");
extern char acledit_export_anchor_3 __asm__("EditPermissionInfo");
extern char acledit_export_anchor_4 __asm__("FMExtensionProcW");
extern char acledit_export_anchor_5 __asm__("SedDiscretionaryAclEditor");
extern char acledit_export_anchor_6 __asm__("SedSystemAclEditor");
extern char acledit_export_anchor_7 __asm__("SedTakeOwnership");
__attribute__((used)) static const void *const acledit_export_anchors[] = {
    &acledit_export_anchor_0,
    &acledit_export_anchor_1,
    &acledit_export_anchor_2,
    &acledit_export_anchor_3,
    &acledit_export_anchor_4,
    &acledit_export_anchor_5,
    &acledit_export_anchor_6,
    &acledit_export_anchor_7,
    0,
};
#endif
