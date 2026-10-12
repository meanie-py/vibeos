
# The HID report-descriptor parser is Rust (sdk/lib/rust/hidparser-rust); the C stays on
# disk and no target compiles it. hidparse.sys links the crate's default (kernel) build,
# hid.dll a `user` variant of it (dll/win32/hid). Its sources were:
#   context.c hidparser.c parser.c api.c hidp.c (+ parser.h, hidp.h)
# They were built twice, as hidparser_um (ntdll) and hidparser_km (ntoskrnl), from one
# source list.
