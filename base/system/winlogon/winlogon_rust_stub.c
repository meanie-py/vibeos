/*
 * Cohabitation stub for the Rust winlogon leaves (winlogon-rust).
 *
 * PARTIAL flip: only the GINA-independent leaf files move to Rust, one
 * at a time. Each flipped .c is commented out of add_executable() (see
 * CMakeLists.txt); its functions are provided by libwinlogon_rust.a
 * (sdk/lib/rust/winlogon-rust/). The still-C winlogon core (winlogon.c/
 * wlx.c/sas.c/...) calls the Rust leaf functions and owns the shared
 * `WLSession` global, which the Rust leaves read through winlogon-sys
 * (WLSESSION layout size-asserted x3).
 *
 * --allow-multiple-definition covers the swap-in window (lld first-match
 * wins; the Rust archive provides the live copy once the C .c is gone).
 * This stub keeps add_executable() valid if a flip ever leaves no other
 * winlogon .c referencing the archive.
 *
 * Currently flipped: setup.c (GetSetupType / RunSetup),
 *                     environment.c (CreateUserEnvironment),
 *                     shutdown.c (StartSystemShutdown / TerminateSystemShutdown),
 *                     screensaver.c (InitializeScreenSaver / StartScreenSaver),
 *                     security.c (Create*Security / Allow*AccessToUser / AllowAccessOnSession),
 *                     notify.c (InitNotifications / CleanupNotifications / CallNotificationDlls).
 *
 * Keep the C on disk (translation philosophy: never delete the C); it is
 * simply not compiled while the Rust translation is active.
 */
