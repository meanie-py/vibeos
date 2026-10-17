/* Cohabitation placeholder TU for the sysaudio kernelmodedriver MODULE.
 *
 * sysaudio's five TUs (control.c, deviface.c, dispatcher.c, main.c, pin.c) are translated
 * to Rust (sdk/lib/rust/sysaudio-rust, linked as libsysaudio_rust.a) and kept on disk per
 * the never-delete rule. DriverEntry and every routine come from the Rust staticlib; the
 * Io/Ks/Ex entry points stay ntoskrnl/ks imports. cmake keeps the MODULE link
 * (set_module_type kernelmodedriver), which needs one C source: this one.
 */
typedef int sysaudio_rust_cohab_placeholder;
