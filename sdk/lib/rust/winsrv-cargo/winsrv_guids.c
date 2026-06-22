/* WS5 cmake-off provider for the COM GUIDs winsrv references — replaces the
 * `sdk/lib/uuid/libuuid.a` (midl-generated `*_i.c`) reuse in winsrv's link.
 *
 * winsrv creates shell shortcuts (the .lnk for the start-menu / desktop), so
 * it pulls exactly THREE GUIDs out of the 2698-symbol libuuid.a — confirmed by
 * an offline link drop-test (drop the 3 cmake static libs, capture lld's
 * undefined list): CLSID_ShellLink, IID_IShellLinkW, IID_IPersistFile.
 *
 * Values are verbatim from the midl output (objidl_i.c / shobjidl_i.c) — the
 * standard, frozen Windows shell GUIDs. The {l,w1,w2,{b1..b8}} layout +
 * `selectany` (COMDAT) match `MIDL_DEFINE_GUID`'s INITGUID expansion
 * (`EXTERN_C const GUID name DECLSPEC_SELECTANY = {...}`) exactly, so each
 * emitted symbol is byte- and section-identical to libuuid.a's; the linker GCs
 * the unreferenced ones just as it does the other 2695 in the cmake build.
 *
 * Self-contained (no headers) so it compiles via reactos-link's `c_shims`
 * (headerless cc-rs) — no cmake / compile_commands.json. The full libuuid
 * cmake-off (self-hosted midl regenerating every `*_i.c`) is the broader
 * "other static_libs" floor; winsrv needs only these three.
 */

typedef struct _WS5_GUID {
    unsigned long Data1;
    unsigned short Data2;
    unsigned short Data3;
    unsigned char Data4[8];
} WS5_GUID;

#define WS5_DEFINE_GUID(name, l, w1, w2, b1, b2, b3, b4, b5, b6, b7, b8) \
    __attribute__((selectany)) const WS5_GUID name = \
        {l, w1, w2, {b1, b2, b3, b4, b5, b6, b7, b8}}

/* clang adds the i386 leading-underscore decoration automatically; amd64/arm64
 * use the bare name — matching libuuid.a's per-arch symbols. */
WS5_DEFINE_GUID(CLSID_ShellLink,  0x00021401, 0x0000, 0x0000,
                0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46);
WS5_DEFINE_GUID(IID_IShellLinkW,  0x000214f9, 0x0000, 0x0000,
                0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46);
WS5_DEFINE_GUID(IID_IPersistFile, 0x0000010b, 0x0000, 0x0000,
                0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46);
