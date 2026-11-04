/* esent is Rust (sdk/lib/rust/esent-rust); a CMake target needs one TU. */
typedef int esent_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char esent_export_anchor_0 __asm__("_JetAttachDatabase2W@16");
extern char esent_export_anchor_1 __asm__("_JetBeginSessionW@16");
extern char esent_export_anchor_2 __asm__("_JetCloseTable@8");
extern char esent_export_anchor_3 __asm__("_JetCreateInstanceW@8");
extern char esent_export_anchor_4 __asm__("_JetGetTableColumnInfoW@24");
extern char esent_export_anchor_5 __asm__("_JetInit@4");
extern char esent_export_anchor_6 __asm__("_JetMove@16");
extern char esent_export_anchor_7 __asm__("_JetOpenDatabaseW@20");
extern char esent_export_anchor_8 __asm__("_JetOpenTableW@28");
extern char esent_export_anchor_9 __asm__("_JetRetrieveColumn@32");
extern char esent_export_anchor_10 __asm__("_JetSetSystemParameterW@20");
extern char esent_export_anchor_11 __asm__("_JetTerm@4");
__attribute__((used)) static const void *const esent_export_anchors[] = {
    &esent_export_anchor_0,
    &esent_export_anchor_1,
    &esent_export_anchor_2,
    &esent_export_anchor_3,
    &esent_export_anchor_4,
    &esent_export_anchor_5,
    &esent_export_anchor_6,
    &esent_export_anchor_7,
    &esent_export_anchor_8,
    &esent_export_anchor_9,
    &esent_export_anchor_10,
    &esent_export_anchor_11,
    0,
};
#elif defined(__x86_64__)
extern char esent_export_anchor_0 __asm__("JetAttachDatabase2W");
extern char esent_export_anchor_1 __asm__("JetBeginSessionW");
extern char esent_export_anchor_2 __asm__("JetCloseTable");
extern char esent_export_anchor_3 __asm__("JetCreateInstanceW");
extern char esent_export_anchor_4 __asm__("JetGetTableColumnInfoW");
extern char esent_export_anchor_5 __asm__("JetInit");
extern char esent_export_anchor_6 __asm__("JetMove");
extern char esent_export_anchor_7 __asm__("JetOpenDatabaseW");
extern char esent_export_anchor_8 __asm__("JetOpenTableW");
extern char esent_export_anchor_9 __asm__("JetRetrieveColumn");
extern char esent_export_anchor_10 __asm__("JetSetSystemParameterW");
extern char esent_export_anchor_11 __asm__("JetTerm");
__attribute__((used)) static const void *const esent_export_anchors[] = {
    &esent_export_anchor_0,
    &esent_export_anchor_1,
    &esent_export_anchor_2,
    &esent_export_anchor_3,
    &esent_export_anchor_4,
    &esent_export_anchor_5,
    &esent_export_anchor_6,
    &esent_export_anchor_7,
    &esent_export_anchor_8,
    &esent_export_anchor_9,
    &esent_export_anchor_10,
    &esent_export_anchor_11,
    0,
};
#elif defined(__aarch64__)
extern char esent_export_anchor_0 __asm__("JetAttachDatabase2W");
extern char esent_export_anchor_1 __asm__("JetBeginSessionW");
extern char esent_export_anchor_2 __asm__("JetCloseTable");
extern char esent_export_anchor_3 __asm__("JetCreateInstanceW");
extern char esent_export_anchor_4 __asm__("JetGetTableColumnInfoW");
extern char esent_export_anchor_5 __asm__("JetInit");
extern char esent_export_anchor_6 __asm__("JetMove");
extern char esent_export_anchor_7 __asm__("JetOpenDatabaseW");
extern char esent_export_anchor_8 __asm__("JetOpenTableW");
extern char esent_export_anchor_9 __asm__("JetRetrieveColumn");
extern char esent_export_anchor_10 __asm__("JetSetSystemParameterW");
extern char esent_export_anchor_11 __asm__("JetTerm");
__attribute__((used)) static const void *const esent_export_anchors[] = {
    &esent_export_anchor_0,
    &esent_export_anchor_1,
    &esent_export_anchor_2,
    &esent_export_anchor_3,
    &esent_export_anchor_4,
    &esent_export_anchor_5,
    &esent_export_anchor_6,
    &esent_export_anchor_7,
    &esent_export_anchor_8,
    &esent_export_anchor_9,
    &esent_export_anchor_10,
    &esent_export_anchor_11,
    0,
};
#endif
