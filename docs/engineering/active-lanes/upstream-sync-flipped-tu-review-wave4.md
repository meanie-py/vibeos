# Upstream-sync: flipped-TU review — wave 4

Scope: wave 3's two explicitly-flagged, unopened clusters — **netcfgx UI-state bugs** (basic +
alternate TCP/IP property dialogs) and **desk cpl icon-sizing / wallpaper-deletion fixes** —
plus a calibration sample of ~9 other items wave 2/3 had triaged at subject/diffstat level only,
picked to answer the standing question: can that remaining triage be trusted?
`FP=5213cf717cff6946945922e6a7e03d87b0a58e4f`, `dll/win32/netcfgx/tcpipconf_notify.c` and
`dll/cpl/desk/{background,effappdlg}.c` cross-checked against `sdk/lib/rust/netcfgx-rust/` and
`sdk/lib/rust/desk-rust/`.

## Registry caveat, hit again

Every C file touched this pass reports `"rust_files": []` in `tools/cohab-registry.json`
despite a real, populated Rust twin existing on disk:

| c_path | registry `rust_files` | actual Rust file |
|---|---|---|
| `dll/win32/netcfgx/tcpipconf_notify.c` | `[]` | `sdk/lib/rust/netcfgx-rust/src/netcfgx/tcpipconf_notify.rs` |
| `dll/cpl/desk/effappdlg.c` | `[]` | `sdk/lib/rust/desk-rust/src/desk/effappdlg.rs` |
| `dll/cpl/desk/appearance.c` | `[]` | `sdk/lib/rust/desk-rust/src/desk/appearance.rs` |
| `dll/cpl/desk/background.c` | `[]` | `sdk/lib/rust/desk-rust/src/desk/background.rs` |
| `dll/cpl/desk/theme.c` | `[]` | (not needed this pass) |

By contrast `dll/win32/ws2_32/src/addrconv.c` (checked as part of the calibration sample) has a
correct, populated `rust_files` entry. So the `dir-name-guess` fallback gap is systematic for
at least netcfgx and desk, not a one-off — worth fixing in the generator (these crates' Rust
trees clearly exist and are laid out predictably at
`sdk/lib/rust/<crate>/src/<c-dir-basename>/<file>.rs`).

## CONFIRMED — detail + proposed diffs

### W4-1. netcfgx basic TCP/IP dialog: radio buttons set with `CheckDlgButton`, stale gateway/DNS survive a toggle to DHCP

- **Files / commits**: `dll/win32/netcfgx/tcpipconf_notify.c` —
  `0df3463af02` ("Fix radio button selection on the basic property page"),
  `5a147d292d6` ("Fix: Default gateway data \"forgotten\" when toggling use of DHCP"),
  `1a04259951e` ("Fix gateway and name server data management in the basic dialog"). All three
  land on the *same* two functions and are best read together as one cluster; our fork predates
  all of them (`FP` sits below `5a147d292d6`, the oldest of the three).
- **Upstream diffs** (net effect, `InitializeTcpipBasicDlgCtrls`):
  1. `CheckDlgButton(hwndDlg, IDC_USEDHCP/IDC_NODHCP, BST_CHECKED)` → `CheckRadioButton(hwndDlg,
     IDC_USEDHCP, IDC_NODHCP, ...)`, same for `IDC_AUTODNS`/`IDC_FIXEDDNS`. `CheckDlgButton` only
     sets/clears the one named button; it does not clear the sibling in the radio group the way
     `CheckRadioButton` does, so the dialog can show both (or, after certain init orders, neither)
     of a mutually-exclusive pair checked.
  2. The `IDC_DEFGATEWAY` "set current gateway" block is moved from *after* (unconditional,
     runs regardless of DHCP/static) to *inside* the static (`else`/`IDC_NODHCP`) branch only —
     so a stale gateway is no longer written into a control that's about to be disabled for DHCP.
  3. `AutoconfigActive` alone no longer drives the DNS radio; a new `DnsDhcpEnabled` field
     (computed once at load, `pSettings->DnsDhcpEnabled = pSettings->DhcpEnabled &&
     pSettings->Ns == NULL`) is threaded through so the "automatic DNS" radio reflects what the
     user actually chose, not just whether any DNS server happens to be configured.
  And in `StoreTcpipBasicSettings` (the WM_COMMAND-driven "commit dialog to model" function):
  toggling to `IDC_USEDHCP` used to leave the previous static IP/gateway pointers alone (only the
  `DhcpEnabled` flag flipped) and *still* read+store whatever value happened to be sitting in the
  (now-disabled) `IDC_DEFGATEWAY` control unconditionally, every time, DHCP or not. The fix moves
  the gateway read/store inside the `IDC_NODHCP` branch and adds explicit list-node frees
  (`This->pCurrentConfig->Ip`, `->Gw`, `->Ns`) when the user switches back to DHCP/automatic DNS.
- **Our Rust (pre-fix, live)**:
  - `sdk/lib/rust/netcfgx-rust/src/netcfgx/tcpipconf_notify.rs:2401-2439`
    (`InitializeTcpipBasicDlgCtrls`) — `CheckDlgButton` (not `CheckRadioButton`) at lines 2402,
    2408, 2421, 2425; the `IDC_DEFGATEWAY` "set current gateway" call at line 2416-2418 sits
    *outside* and *after* the `if/else` on `DhcpEnabled`, run unconditionally exactly like the
    pre-fix C; the DNS radio at line 2420 keys off `(*pCurSettings).AutoconfigActive != 0` alone
    — there is no `DnsDhcpEnabled` field anywhere in the `TcpipSettings` struct in this crate.
  - `sdk/lib/rust/netcfgx-rust/src/netcfgx/tcpipconf_notify.rs:2205-2369`
    (`StoreTcpipBasicSettings`) — the `IDC_DEFGATEWAY` `IPM_GETADDRESS`/store block
    (lines 2276-2298) sits *outside* the `IsDlgButtonChecked(IDC_NODHCP)` if/else (lines
    2212-2274), i.e. it runs unconditionally on every Apply/OK regardless of the DHCP/static
    choice, byte-for-byte the pre-`1a04259951e` shape; the `else` branch that sets
    `DhcpEnabled = TRUE` (line 2273) does not free `Ip`/`Gw`; the `AutoconfigActive = TRUE` else
    branch (line 2367) does not free `Ns`.
- Both functions are `#[no_mangle] pub unsafe extern "C"`, called directly from the property
  sheet's `TcpipBasicDlgProc` (`WM_INITDIALOG` / `WM_COMMAND` → `PSN_APPLY`) — live, reachable
  by any user opening a network adapter's TCP/IPv4 Properties "General" tab. **User-visible
  effect**: (a) the DHCP/Static and Automatic-DNS/Fixed-DNS radio pairs can render with the
  wrong (or an ambiguous) selection on dialog open, matching upstream's own bug title; (b) a
  gateway typed while in Static mode is silently retained and re-applied even after switching
  back to DHCP, i.e. exactly the "forgotten" data upstream's commit title names, except here it
  is *not* forgotten when it should be; (c) switching to DHCP or Automatic DNS leaks the old
  `IP_ADDR` list nodes (`Ip`/`Gw`/`Ns`) instead of freeing them — a small but real per-Apply
  memory leak on top of the stale-data bug.
- Proposed fix (three independent hunks; shown against the actual line numbers above):
```diff
--- a/sdk/lib/rust/netcfgx-rust/src/netcfgx/tcpipconf_notify.rs
+++ b/sdk/lib/rust/netcfgx-rust/src/netcfgx/tcpipconf_notify.rs
@@ InitializeTcpipBasicDlgCtrls
     if (*pCurSettings).DhcpEnabled != 0 {
-        CheckDlgButton(hwndDlg, IDC_USEDHCP, BST_CHECKED);
+        CheckRadioButton(hwndDlg, IDC_USEDHCP, IDC_NODHCP, IDC_USEDHCP);
         EnableWindow(GetDlgItem(hwndDlg, IDC_IPADDR), (FALSE) as i32);
         EnableWindow(GetDlgItem(hwndDlg, IDC_SUBNETMASK), (FALSE) as i32);
         EnableWindow(GetDlgItem(hwndDlg, IDC_DEFGATEWAY), (FALSE) as i32);
         EnableWindow(GetDlgItem(hwndDlg, IDC_AUTODNS), (TRUE) as i32);
     } else {
-        CheckDlgButton(hwndDlg, IDC_NODHCP, BST_CHECKED);
+        CheckRadioButton(hwndDlg, IDC_USEDHCP, IDC_NODHCP, IDC_NODHCP);
 
         if !(*pCurSettings).Ip.is_null() {
             SendDlgItemMessageA(hwndDlg, IDC_IPADDR, (IPM_SETADDRESS) as u32, 0, (*(*pCurSettings).Ip).IpAddress as LPARAM);
             SendDlgItemMessageA(hwndDlg, IDC_SUBNETMASK, (IPM_SETADDRESS) as u32, 0, (*(*pCurSettings).Ip).u.Subnetmask as LPARAM);
         }
+
+        if !(*pCurSettings).Gw.is_null() && (*(*pCurSettings).Gw).IpAddress != 0 {
+            SendDlgItemMessageA(hwndDlg, IDC_DEFGATEWAY, (IPM_SETADDRESS) as u32, 0, (*(*pCurSettings).Gw).IpAddress as LPARAM);
+        }
     }
 
-    if !(*pCurSettings).Gw.is_null() && (*(*pCurSettings).Gw).IpAddress != 0 {
-        SendDlgItemMessageA(hwndDlg, IDC_DEFGATEWAY, (IPM_SETADDRESS) as u32, 0, (*(*pCurSettings).Gw).IpAddress as LPARAM);
-    }
-
-    if (*pCurSettings).AutoconfigActive != 0 {
-        CheckDlgButton(hwndDlg, IDC_AUTODNS, BST_CHECKED);
+    if (*pCurSettings).DnsDhcpEnabled != 0 {
+        CheckRadioButton(hwndDlg, IDC_AUTODNS, IDC_FIXEDDNS, IDC_AUTODNS);
         EnableWindow(GetDlgItem(hwndDlg, IDC_DNS1), (FALSE) as i32);
         EnableWindow(GetDlgItem(hwndDlg, IDC_DNS2), (FALSE) as i32);
     } else {
-        CheckDlgButton(hwndDlg, IDC_FIXEDDNS, BST_CHECKED);
+        CheckRadioButton(hwndDlg, IDC_AUTODNS, IDC_FIXEDDNS, IDC_FIXEDDNS);
         EnableWindow(GetDlgItem(hwndDlg, IDC_DNS1), (TRUE) as i32);
         EnableWindow(GetDlgItem(hwndDlg, IDC_DNS2), (TRUE) as i32);
         ...
```
  (`DnsDhcpEnabled` must be added as a field of `TcpipSettings` and computed at load time,
  mirroring upstream's `pSettings->DnsDhcpEnabled = pSettings->DhcpEnabled && pSettings->Ns ==
  NULL` in `LoadTcpIpSettings` — find that function in the same file before wiring this in, it
  is not shown here to keep the diff focused.)
```diff
--- a/sdk/lib/rust/netcfgx-rust/src/netcfgx/tcpipconf_notify.rs
+++ b/sdk/lib/rust/netcfgx-rust/src/netcfgx/tcpipconf_notify.rs
@@ StoreTcpipBasicSettings
     if IsDlgButtonChecked(hwndDlg, IDC_NODHCP) == BST_CHECKED as UINT {
         (*(*This).pCurrentConfig).DhcpEnabled = FALSE as DWORD;
         ... (existing IP/subnet store) ...
         (*(*(*This).pCurrentConfig).Ip).u.Subnetmask = dwIpAddr;
+
+        if SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, (IPM_GETADDRESS) as u32, 0, &mut dwIpAddr as *mut _ as LPARAM) == 4 {
+            if (*(*This).pCurrentConfig).Gw.is_null() {
+                (*(*This).pCurrentConfig).Gw = CoTaskMemAlloc((core::mem::size_of::<IP_ADDR>()) as SIZE_T) as *mut IP_ADDR;
+                if (*(*This).pCurrentConfig).Gw.is_null() { return (E_OUTOFMEMORY) as LRESULT; }
+                ZeroMemory!((*(*This).pCurrentConfig).Gw as *mut _, core::mem::size_of::<IP_ADDR>());
+            }
+            (*(*(*This).pCurrentConfig).Gw).IpAddress = dwIpAddr;
+        }
     } else {
         (*(*This).pCurrentConfig).DhcpEnabled = TRUE as DWORD;
+
+        if !(*(*This).pCurrentConfig).Ip.is_null() {
+            let pNextIp = (*(*(*This).pCurrentConfig).Ip).Next;
+            CoTaskMemFree((*(*This).pCurrentConfig).Ip as *mut _);
+            (*(*This).pCurrentConfig).Ip = pNextIp;
+        }
+        if !(*(*This).pCurrentConfig).Gw.is_null() {
+            let pNextGw = (*(*(*This).pCurrentConfig).Gw).Next;
+            CoTaskMemFree((*(*This).pCurrentConfig).Gw as *mut _);
+            (*(*This).pCurrentConfig).Gw = pNextGw;
+        }
     }
 
-    if SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, (IPM_GETADDRESS) as u32, 0, &mut dwIpAddr as *mut _ as LPARAM) == 4 {
-        ... (existing unconditional gateway store/free, lines 2276-2298) ...
-    }
-
     if IsDlgButtonChecked(hwndDlg, IDC_FIXEDDNS) == BST_CHECKED as UINT {
         ... (unchanged) ...
     } else {
         (*(*This).pCurrentConfig).AutoconfigActive = TRUE as DWORD;
+        if !(*(*This).pCurrentConfig).Ns.is_null() {
+            let pNextNs = (*(*(*This).pCurrentConfig).Ns).Next;
+            CoTaskMemFree((*(*This).pCurrentConfig).Ns as *mut _);
+            (*(*This).pCurrentConfig).Ns = pNextNs;
+        }
     }
```
  (This deletes the old unconditional `IDC_DEFGATEWAY` block wholesale and re-adds its body
  inside the `IDC_NODHCP` arm — check the exact surrounding brace structure against the live
  file before applying, the line numbers above are from the read at the time of this review.)

### W4-2. netcfgx alternate-configuration dialog: gateway/DNS written and DNS controls enabled independent of which radio (DHCP-fallback-to-APIPA vs. user config) is selected

- **File / commit**: `dll/win32/netcfgx/tcpipconf_notify.c`, `4b75ec5508d`
  ("Fix alternate configuration TCP/IP dialog — Gateway and name server ip address controls
  must only be enabled and initialized when a non-APIPA configuration has been selected").
- **Upstream diff**: in `InitializeTcpipAltDlgCtrls`, moves the unconditional
  `EnableWindow(IDC_DNS1/2, TRUE)`, the `IDC_DEFGATEWAY` "set current gateway" block, and the
  `IDC_DNS1/2` "set name server" block from *after* the DHCP/NODHCP `if/else` to *inside* the
  `NODHCP` (user-configured, non-APIPA) branch only; also stops unconditionally
  `IPM_CLEARADDRESS`-ing DNS1/DNS2 when `pCurSettings->Ns` is null (that clear now only happens
  implicitly by not touching the controls, since they stay disabled+blank from range-set), and
  adds `IPM_CLEARADDRESS` for DNS1/DNS2 in `TcpipAltConfDlg`'s toggle handler (previously only
  IP/subnet/gateway were cleared on toggle-to-DHCP, not DNS).
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/netcfgx-rust/src/netcfgx/tcpipconf_notify.rs:2062-2127`
  (`InitializeTcpipAltDlgCtrls`) — line 2101-2102 unconditionally enables `IDC_DNS1`/`IDC_DNS2`
  inside the `else` (NODHCP) branch already (matches an *intermediate*, not final, upstream
  state — see below), but the gateway block (2110-2112) and the Ns block (2114-2124) both sit
  **after and outside** the `if/else`, running regardless of whether DHCP (APIPA-fallback) or
  static was selected — exactly the pre-fix shape upstream's commit message calls out by name.
  The toggle handler `TcpipAltConfDlg` (line 2151-2172) clears only `IDC_IPADDR`,
  `IDC_SUBNETMASK`, `IDC_DEFGATEWAY` on toggle-to-DHCP (line 2159-2161) — DNS1/DNS2 are not
  cleared, matching the pre-fix C exactly.
- `#[no_mangle] pub unsafe extern "C" fn InitializeTcpipAltDlgCtrls` /
  `#[no_mangle] pub unsafe extern "system" fn TcpipAltConfDlg` — live, the WM_INITDIALOG/
  WM_COMMAND handlers for the "Alternate Configuration" tab of TCP/IPv4 Properties (visible only
  when the main tab has DHCP selected). **User-visible effect**: switching the alternate tab's
  radio to "Automatic private IP address" (APIPA) still shows/holds stale gateway and DNS server
  values from a previous "User configured" selection in the (now-disabled) controls, and
  clicking back and forth leaves DNS1/DNS2 un-cleared where IP/gateway are cleared — an
  inconsistency a user could notice as "my DNS servers didn't reset like the IP did."
- Proposed fix:
```diff
--- a/sdk/lib/rust/netcfgx-rust/src/netcfgx/tcpipconf_notify.rs
+++ b/sdk/lib/rust/netcfgx-rust/src/netcfgx/tcpipconf_notify.rs
@@ InitializeTcpipAltDlgCtrls
         if !(*pCurSettings).Ip.is_null() {
             SendDlgItemMessageA(hwndDlg, IDC_IPADDR, (IPM_SETADDRESS) as u32, 0, (*(*pCurSettings).Ip).IpAddress as LPARAM);
             SendDlgItemMessageA(hwndDlg, IDC_SUBNETMASK, (IPM_SETADDRESS) as u32, 0, (*(*pCurSettings).Ip).u.Subnetmask as LPARAM);
         }
-    }
 
-    if !(*pCurSettings).Gw.is_null() && (*(*pCurSettings).Gw).IpAddress != 0 {
-        SendDlgItemMessageA(hwndDlg, IDC_DEFGATEWAY, (IPM_SETADDRESS) as u32, 0, (*(*pCurSettings).Gw).IpAddress as LPARAM);
-    }
+        if !(*pCurSettings).Gw.is_null() && (*(*pCurSettings).Gw).IpAddress != 0 {
+            SendDlgItemMessageA(hwndDlg, IDC_DEFGATEWAY, (IPM_SETADDRESS) as u32, 0, (*(*pCurSettings).Gw).IpAddress as LPARAM);
+        }
 
-    if !(*pCurSettings).Ns.is_null() {
-        SendDlgItemMessageW(hwndDlg, IDC_DNS1, (IPM_SETADDRESS) as u32, 0, (*(*pCurSettings).Ns).IpAddress as LPARAM);
-        if !(*(*pCurSettings).Ns).Next.is_null() {
-            SendDlgItemMessageW(hwndDlg, IDC_DNS2, (IPM_SETADDRESS) as u32, 0, (*(*(*pCurSettings).Ns).Next).IpAddress as LPARAM);
-        } else {
-            SendDlgItemMessageW(hwndDlg, IDC_DNS2, (IPM_CLEARADDRESS) as u32, 0, 0);
+        if !(*pCurSettings).Ns.is_null() {
+            SendDlgItemMessageW(hwndDlg, IDC_DNS1, (IPM_SETADDRESS) as u32, 0, (*(*pCurSettings).Ns).IpAddress as LPARAM);
+            if !(*(*pCurSettings).Ns).Next.is_null() {
+                SendDlgItemMessageW(hwndDlg, IDC_DNS2, (IPM_SETADDRESS) as u32, 0, (*(*(*pCurSettings).Ns).Next).IpAddress as LPARAM);
+            }
         }
-    } else {
-        SendDlgItemMessageW(hwndDlg, IDC_DNS1, (IPM_CLEARADDRESS) as u32, 0, 0);
-        SendDlgItemMessageW(hwndDlg, IDC_DNS2, (IPM_CLEARADDRESS) as u32, 0, 0);
     }
@@ TcpipAltConfDlg (WM_COMMAND / IDC_USEDHCP | IDC_NODHCP)
                         if bNoDHCP != 0 {
                             SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_CLEARADDRESS) as u32, 0, 0);
                             SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, (IPM_CLEARADDRESS) as u32, 0, 0);
                             SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, (IPM_CLEARADDRESS) as u32, 0, 0);
+                            SendDlgItemMessageW(hwndDlg, IDC_DNS1, (IPM_CLEARADDRESS) as u32, 0, 0);
+                            SendDlgItemMessageW(hwndDlg, IDC_DNS2, (IPM_CLEARADDRESS) as u32, 0, 0);
                         }
```

### W4-3. `ws2_32.dll` `inet_addr`: out-of-bounds stack write on 5+ dot-separated input — the exact bug upstream's commit says caused stack corruption

- **File / commit**: `dll/win32/ws2_32/src/addrconv.c`, `6679825f21e` ("[WS2_32] Import inet_addr
  from wine" — commit message: *"The old implementation was buggy and caused stack corruption
  (triggered by ws2_32_winetest:sock test_inet_pton)"*). This was in wave 3's "not reached" list,
  filed under "subjects don't read as fixes to existing behavior" — **that characterization is
  wrong; the subject undersells a real memory-safety bug, and this is the flip of this pass.**
- **Upstream bug mechanics** (pre-fix `addrconv.c`, ported from the classic BSD `inet_aton`):
  a local `u_long parts[4]` and cursor `pp` walk dot-separated numeric groups. Inside the loop,
  each group parsed *before* a `.` is bounds-checked (`if (pp >= parts + 4) return INADDR_NONE;`)
  before `*pp++ = val`. But the group parsed *after* the last `.` (i.e. not followed by another
  `.`) is written **unconditionally**, with no re-check: `*pp++ = val;` right before the `switch
  (n)` that builds the final address. An input with 5+ dot-separated groups (e.g.
  `"1.2.3.4.5"`) passes the bounds check on groups 1-4 (`pp` reaches `parts+4`), then the 5th
  group's trailing unconditional write lands at `parts[4]` — one `u_long` past the end of a
  4-element stack array. Upstream's fix replaces the whole function with Wine's `strtoul`-based
  implementation, which loops `for (i = 0; i < 4; ++i)` and never writes past `a[3]`.
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/ws2_32-rust/src/src/addrconv.rs:162-226`,
  `inet_addr` — a direct, faithful port of the same BSD algorithm, including the same bug:
  line 212-214 has the same in-loop bounds check (`if pp >= parts.as_mut_ptr().offset(4) {
  return INADDR_NONE; }`) guarding the dot-continuation write at line 215-217, but the trailing
  write at **line 223-224** (`*pp = val; pp = pp.offset(1);`) is unconditional, exactly mirroring
  the upstream bug — this is a raw-pointer write through `*mut u_long` with **no bounds check at
  all**, so `inet_addr("1.2.3.4.5")` writes one `u_long` (4 or 8 bytes depending on target)
  immediately past the 16-byte `parts: [u_long; 4]` stack array, i.e. genuine stack corruption
  in unsafe Rust, identical in kind (and root cause) to the C bug upstream named.
- `#[no_mangle] pub unsafe extern "system" fn inet_addr` — live, directly exported from
  `ws2_32.dll` (`inet_addr` is `ws2_32.11` per the ordinal comment in Wine's replacement), called
  by essentially any Winsock client that accepts a dotted-quad-ish string from a config file,
  command line, or network peer and needs a `u_long` address (many legacy apps still call it
  instead of `inet_pton`). **This is the highest-severity finding of this pass**: it is a
  stack-buffer write with attacker-influenced offset-adjacent content, triggerable with a single
  malformed string, in a DLL loaded by any networked process — no test harness or special
  reachability needed.
- Proposed fix — the **minimal, behavior-preserving** guard (adds the same bounds check already
  used one line above it, rather than adopting upstream's larger Wine-import rewrite, to keep
  this a surgical memory-safety fix rather than a behavior-changing replacement):
```diff
--- a/sdk/lib/rust/ws2_32-rust/src/src/addrconv.rs
+++ b/sdk/lib/rust/ws2_32-rust/src/src/addrconv.rs
@@ pub unsafe extern "system" fn inet_addr
     if *cp as i32 != 0 && isspace(*cp as i32) == 0 {
         return INADDR_NONE;
     }
 
+    if pp >= parts.as_mut_ptr().offset(4) {
+        return INADDR_NONE;
+    }
     *pp = val;
     pp = pp.offset(1);
```
  Note for whoever applies this: upstream chose instead to import Wine's `strtoul`-based
  rewrite wholesale (also fixes a handful of other edge cases — leading-space handling, explicit
  per-octet `> 255` range checks that the BSD-derived version never had). That is a strictly
  better long-term replacement and worth doing as a deliberate follow-up, but it changes
  behavior on more inputs than just the crashing one; the guard above closes the memory-safety
  hole with a one-line, behavior-neutral-for-all-valid-input change, appropriate for landing
  immediately without a wider review.

### W4-4. `desk.cpl` background tab: a deleted wallpaper file still gets shown as a valid entry (with a leaked icon handle)

- **File / commits**: `dll/cpl/desk/background.c`, `3016c40dc72` ("Check if wallpaper file is
  deleted (#8670)" / CORE-19896) followed by its own partial revert `249be9ecdd3` ("Follow-up of
  #8670; Revert item selection code" / CORE-20499). Net effect from `FP` to current upstream
  (the revert only undoes the `LVIS_SELECTED` reordering, not the substance):
  1. `SHGetFileInfoW(wallpaperFilename, ...)`'s return value is now checked; on failure (file
     doesn't exist — user deleted the wallpaper file after Windows/ReactOS recorded it as the
     current one) the function does `RegCloseKey(regKey); return;` instead of proceeding.
  2. `DestroyIcon(sfi.hIcon)` is added right after `ImageList_AddIcon(himl, sfi.hIcon)` — the
     image list makes its own copy, so the handle `SHGetFileInfoW` allocated must be destroyed
     or it leaks.
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/desk-rust/src/desk/background.rs:349-356`,
  `AddListViewItems` — `SHGetFileInfoW(...)`'s return value is discarded (line 349-355, no `if`
  around it at all); no `DestroyIcon` call anywhere near `sfi.hIcon` (`ImageList_AddIcon!(himl,
  sfi.hIcon)` at line 356 is the only use of that handle). If the registered wallpaper file has
  been deleted, `sfi` stays zero-initialized (`core::mem::zeroed()`, line 257) except for
  whatever the failed call happened to touch, so a bogus/blank list entry is still built,
  `StringCbCopy` from `sfi.szDisplayName` (line 364) copies empty/garbage text, and the entry is
  inserted into the wallpaper list — a broken "ghost" entry for a file that no longer exists,
  which is exactly the symptom CORE-19896 reports upstream. Every call through this code path
  also leaks one icon handle.
- `AddListViewItems` is a private helper (no `#[no_mangle]`) but is called from
  `BackgroundPageProc`'s `WM_INITDIALOG`, which **is** the live, exported dialog proc for the
  Desktop/Background tab of Display Properties — reached every time that tab is opened.
- Proposed fix:
```diff
--- a/sdk/lib/rust/desk-rust/src/desk/background.rs
+++ b/sdk/lib/rust/desk-rust/src/desk/background.rs
@@ AddListViewItems
-            SHGetFileInfoW(
+            if SHGetFileInfoW(
                 wallpaperFilename.as_ptr(),
                 0,
                 (&mut sfi as *mut _ as *mut core::ffi::c_void).cast(),
                 core::mem::size_of_val(&sfi) as u32,
                 SHGFI_ICON | SHGFI_SMALLICON | SHGFI_DISPLAYNAME,
-            );
+            ) == 0
+            {
+                RegCloseKey(regKey);
+                return;
+            }
             sfi.iIcon = ImageList_AddIcon!(himl, sfi.hIcon);
+            DestroyIcon(sfi.hIcon);
 
             i += 1;
```
  (Verify `SHGetFileInfoW`'s Rust binding return type/FFI convention in this crate's prelude —
  the C compares against `0`/`FALSE`; match whatever truthy/falsy convention the binding uses.)

## REFUTED — detail (calibration sample)

- **`dll/cpl/desk/{appearance,effappdlg}.c`, `56746c29df0`** ("Apply the Shell Icon Size
  properly", #9336) — wave 3 flagged this as a plausible real UI-correctness bug worth a look;
  it is not. The pre-fix `IDC_EFFAPPEARANCE_LARGEICONS` checkbox ships `WS_TABSTOP |
  WS_DISABLED` in every localized `.rc` (confirmed in the diff: the fix's own hunk *removes*
  `WS_DISABLED`), and `effappdlg.c` had **zero** code reading or writing it before this commit —
  no `g->SchemeAdv.iIconSize` field use, no `BM_GETCHECK`/`BM_SETCHECK` on that control at all.
  This is standing up a previously-inert, disabled control as a real 3-state feature from
  scratch, not fixing existing logic. Confirmed our Rust twin
  (`sdk/lib/rust/desk-rust/src/desk/effappdlg.rs`) has no `LARGEICONS`/`IconSize` handling
  either (`grep` for both terms: zero hits) — consistent translation-backlog gap, not a
  regression. **This is the one item in this pass where wave 3's own stated suspicion
  ("read as real UI-correctness bugs... worth a future pass") turned out generous** — filed
  here as the flip for that specific claim, distinct from the ws2_32 flip below.
- **`dll/win32/netcfgx/inetcfgcomp_iface.c`, `54c667caaa7`** and
  **`dll/win32/netcfgx/tcpipconf_notify.c`, `a2c594187e5`** (both "Fix calls to wcsicmp") —
  both rename `wcsicmp` → `_wcsicmp`, the MSVC-vs-mingw/POSIX libc symbol-naming difference
  (`wcsicmp` doesn't exist as a linkable symbol under MSVC's CRT; `_wcsicmp` does). Purely a
  C-build portability fix with no behavior change, and not applicable to Rust at all (this
  crate calls `core`/its own libc shim, not either C symbol). REFUTED as a translation-relevant
  bug for both files.
- **`dll/cpl/sysdm/performance.c`, `2290d0e7855`** ("Enhance shlobj_undoc.h for associations",
  #9064) — the file's only hunk is a missing `#include <shlwapi_undoc.h>` to fix a C build after
  a header reorg; all the substantive work is in the two `*_undoc.h` headers (interface/GUID
  additions with no C predecessor behavior). No logic change in `performance.c` itself. REFUTED.
- **`dll/win32/iphlpapi/iphlpapi_main.c`, `7f9e42cfe21`** ("Implement
  NhGetGuidFromInterfaceName") — confirmed by reading the diff: this is a **new** function
  appended after the existing `NhGetInterfaceNameFromGuid`, previously entirely unimplemented.
  Pure feature addition, nothing to regress against. REFUTED as a bug-hunt target (translation
  backlog item instead, consistent with wave 3's read).

## Not reached

- **`7139e57ff89`** ("Fix the alternate network configuration") — touches
  `tcpipconf_notify.c` plus every localized `.rc` (new/renumbered resource IDs for the alternate
  config dialog) and its commit message itself says "TODO: Fix the format of the alternate
  configuration data," i.e. even upstream considered this a partial fix, later superseded by
  `a34fd324ef6` ("Keep two configurations in the TCPIP notification object", a real structural
  change adding a second `TcpipSettings`-shaped struct) which sits *after* it and which we do
  not have. Applying `7139e57ff89` cleanly to our current (pre-`a34fd324ef6`) struct layout is
  not a small diff — flagging as a real candidate but deferring the actual patch to a dedicated
  pass rather than rushing a compound structural change under this review's budget.
- **`afbc9224052`** ("Fix the output of ITcpipProperties_fnUnknown1") — reads as a real
  correctness fix (empty parameter returned as `'(null)'` string instead of `""`, missing
  `DynamicUpdate`/`NameRegistration` output params) but not opened in full this pass; worth a
  dedicated look given it's an actual "Fix" title in a file already known to be cohab-flipped.
- **The rest of wave 3's ~70-file subject-only bucket**: this pass sampled 7 items
  (54c667caaa7, a2c594187e5, 2290d0e7855, 7f9e42cfe21, 6679825f21e, plus the two netcfgx/desk
  clusters that were the primary target) out of that ~70; the remaining ~63
  (netcfgx "Implement X" COM/RPC feature commits, shell32 feature commits, freeldr/ui, acpi,
  remaining cpl/sysdm and dnsrslvr/tcpsvcs/userenv/wshtcpip singles) are still genuinely
  unopened. See wave 3's own "Not reached" section for the itemized list — nothing here
  supersedes it except the two files explicitly re-triaged above (`iphlpapi_main.c`,
  `ws2_32/src/addrconv.c`).
- **Registry generator fix**: not applied (this agent is read-only) — flagging that
  `netcfgx-rust` and `desk-rust`'s `rust_files: []` entries (5 files, table above) should be
  corrected in `tools/cohab-registry.json`'s generator; the pattern
  `sdk/lib/rust/<crate>/src/<c_dir_basename>/<file>.rs` resolved correctly for all 5 by hand.

## Summary

- **Re-checked this pass**: 2 explicitly-flagged clusters (netcfgx dialogs: 5 commits across 2
  functions' worth of combined bugs; desk background/effappdlg: 3 commits) + a 7-commit
  calibration sample from wave 3's subject-only bucket picked for variety (2 build-fix
  "wcsicmp" commits, 1 header-only sysdm commit, 1 iphlpapi feature commit, 1 ws2_32 commit, plus
  the desk icon-size commit already counted above). **12 commits total substantively opened.**
- **Held vs. flipped, on the 7-commit calibration sample plus the desk icon-size item (8
  independently-judged items with a prior subject-level characterization to check against)**:
  **6 held** (54c667caaa7, a2c594187e5, 2290d0e7855, 7f9e42cfe21 all confirmed non-bugs for us
  as their subjects suggested; the netcfgx and desk-wallpaper *clusters* wave 3 flagged as
  "read as real bugs, worth a look" were in fact confirmed real, so that suspicion held too),
  **2 flipped**: `6679825f21e` (ws2_32 `inet_addr`) was filed as "doesn't read as a fix to
  existing behavior" and is in fact a stack-corruption memory-safety bug we carry verbatim —
  the single highest-severity finding of this pass; and `56746c29df0` (desk icon-size) was
  flagged as a plausible real bug and turned out to be a pure feature-add with nothing to
  regress against. **That's 2 flips in 8 judged items (25%)** — on this small sample, subject
  triage is still not reliable enough to skip full-diff verification, consistent with wave 3's
  own finding (7 of 18 refuted) and the two calibration proofs in this task's brief.
- **4 CONFIRMED bugs** (W4-1 through W4-4), all **live**:
  - **W4-3** (`ws2_32.dll`, every networked process): `inet_addr` stack-buffer overflow on
    5+-group dotted input — highest severity, a raw out-of-bounds pointer write triggerable by
    a single malformed string with no special privileges.
  - **W4-1** (netcfgx.dll, Network Connections TCP/IPv4 Properties, "General" tab): radio-button
    UI-state bug + stale gateway/DNS data surviving a DHCP toggle + memory leak on toggle.
  - **W4-2** (netcfgx.dll, same properties sheet, "Alternate Configuration" tab): same class of
    UI-state/stale-data bug, independent code path.
  - **W4-4** (desk.cpl, Display Properties "Desktop" tab): deleted-wallpaper ghost list entry +
    icon-handle leak, low severity but a real, reproducible correctness bug (CORE-19896).
- **4 REFUTED**: two MSVC-only symbol-rename build fixes (N/A to Rust), one header-only sysdm
  build fix, one pure-feature iphlpapi addition, one pure-feature desk icon-size addition —
  see detail above for why each has nothing to regress against.
- **Registry caveat reproduced**: 5 of 5 files opened this pass had `rust_files: []` in the
  registry despite a real, correctly-located Rust twin on disk in every case (netcfgx-rust ×1,
  desk-rust ×4) — this looks like a systematic gap for these two crates specifically, not
  isolated noise; `ws2_32-rust`'s entry, by contrast, was accurate.
