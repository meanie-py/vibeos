# RDP server: remote desktop for headless boards

Status: proposed 2026-10-08. Roadmap: `docs/product/vibeos-roadmap.md`, quest 10g.

The Pi 4, the Pi 5 and Graviton usually run with no monitor, and the remote-desktop
client every Windows machine already has is `mstsc`. So VibeOS should *serve* RDP.
This doc says what RDP is, what the tree already has, and the order to build it in.
The protocol engine will be **IronRDP**.

## What RDP is

Microsoft's Remote Desktop Protocol, TCP (and optionally UDP) port 3389. Unlike VNC it is
not a pixel scrape: the server sends drawing orders and compressed bitmaps (newer servers
use H.264 or RemoteFX/progressive codecs through the graphics pipeline, `MS-RDPEGFX`), and
the client sends keyboard and mouse back. The layers, bottom up:

| Layer | What it does | Spec |
|---|---|---|
| X.224 / T.125 MCS | connection and channel framing, inherited from ITU conferencing | `MS-RDPBCGR` |
| TLS + CredSSP (NLA) | encryption; NLA authenticates the user (NTLM or Kerberos via SSPI) *before* a session is built | `MS-CSSP` |
| Capability exchange | screen size, colour depth, codecs, input types | `MS-RDPBCGR` |
| Fast-path I/O | screen updates out, input in | `MS-RDPBCGR` |
| Virtual channels | clipboard, drives, audio, printers, USB, display resize | `MS-RDPECLIP`, `MS-RDPEFS`, `MS-RDPEA`, ... |

On Windows the server side is a subsystem, not a program: TermService (`termsrv.dll`),
a remote display driver, remote keyboard/mouse drivers (`termdd`), and real multi-session,
where each session (1, 2, ...) has its own win32k session space, csrss and winlogon.

## What the tree has

- **A client.** `base/applications/mstsc` is rdesktop's engine (`vendor/rdesktop`) behind
  ReactOS's front end, so VibeOS already connects *to* RDP servers.
- **No server.** No TermService; `wtsapi32` is stubs; nothing is multi-session.
- **Pieces that help.** Kernel TCP listen with a backlog (netstack `tditrans.rs`); a
  network-login service to imitate (`telnetd`); `schannel` is built on mbedTLS.

## Why IronRDP

[IronRDP](https://github.com/Devolutions/IronRDP) (Devolutions, MIT/Apache-2.0) is a Rust
RDP implementation split into crates: `ironrdp-pdu` (encoding), `ironrdp-connector` /
`ironrdp-acceptor` (state machines), `ironrdp-graphics` (codecs), channel crates
(clipboard, drives, audio, display control), and `ironrdp-server`, a ready server whose
embedder supplies a display source and an input sink. That embedder shape is exactly
tier 1 below, and it is Rust, which is the project's goal. FreeRDP's server library (C)
is the fallback.

The cost to check first: `ironrdp-server` 0.13 is built on **tokio** and
**tokio-rustls**, so it needs Rust `std` on the guest's user-mode target
(`*-pc-windows-gnullvm`). No guest crate uses `std` today. Two ways through:

1. **Bring up `std` for guest user mode** (std on windows-gnullvm calls kernel32, ws2_32,
   bcrypt and ntdll, all of which we ship). The first step of this quest is a `std` hello
   world with a tokio TCP echo, run in the guest. If that works, `ironrdp-server` is used
   as is. This also unlocks every other std crate for user mode.
2. **Use the sans-IO crates directly.** `ironrdp-acceptor`, `-pdu` and `-graphics` are
   I/O-free state machines; drive them from our own Winsock loop and do TLS through
   schannel (mbedTLS) instead of rustls. More glue, no std dependency.

Settle 1 vs 2 by trying 1 first: it is a day of work and its answer is useful anyway.

## Tiers

**0. A display with no monitor.** win32k needs a surface. A headless Pi may get no GOP
framebuffer, and Graviton has no GPU at all. Add a memory-backed display (a miniport whose
"framebuffer" is RAM at a configured mode), selected when firmware hands over no GOP. It is
needed by any remote display, RDP or VNC.

**1. Shadow the console over RDP.** A service (`rdpsvc`) that:
- listens on 3389 and runs the IronRDP acceptor;
- captures the console: read the memory framebuffer from tier 0 and track dirty regions
  (start with periodic full-frame diffing; a display-driver dirty-rect feed later);
- encodes bitmap updates (IronRDP's RLE/planar first, RemoteFX or QOI later);
- injects input with `SendInput` on `WinSta0\Default`, following desktop switches to the
  Winlogon desktop.

Hard parts:
- **Ctrl+Alt+Del** cannot be synthesized with `SendInput` by design. The service needs a
  privileged path that raises the SAS to winlogon (Windows has `SendSAS` for this).
- **Desktop switching**: the service must reattach when the input desktop changes.
- **Security**: start with TLS-only security and let the user log in at winlogon's screen
  (mstsc warns, then connects). NLA needs server-side CredSSP over our NTLM/Kerberos in
  lsass. Owed, not skipped: a box reachable on 3389 without NLA is exposed to anyone who can
  reach the login screen.

This tier is the deliverable: the Pis and Graviton reachable from any Windows machine.

**2. Real Terminal Services (multi-session).** Per-session win32k and session space, csrss
and winlogon per session, a display driver per session. Deep kernel and win32k work that
ReactOS never finished. Only if "several users on one box" becomes a goal.

**3. Channels and codecs.** Clipboard and display resize first, then drive redirection and
audio, then H.264 (the Pi 5 has no hardware encoder: a CPU cost).

## Verification

- QEMU: a host-side IronRDP or FreeRDP client in `tests/qemu` connects through a slirp
  hostfwd to 3389, logs in, and checks a frame for the desktop marker. No `mstsc` needed
  in CI.
- Boards: `mstsc` from the Windows host to the Pi's lease address.
