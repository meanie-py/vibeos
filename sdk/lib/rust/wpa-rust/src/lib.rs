//! WPA2-Personal: the station side of the 4-way and group-key handshakes (IEEE 802.11-2016
//! 12.7), with the SHA-1 and AES-128 they need. Shared by ndis-rust's WDI layer (the
//! OptiPlex's Realtek, `ndis::wpa`) and bwfm-rust (the Pis' CYW43455, whose firmware leaves
//! the handshake to the host with `sup_wpa 0`).
//!
//! Not a translation. On Windows the supplicant lives in user mode (wlansvc and its
//! security modules) and hands nwifi.sys finished keys; ReactOS has neither. This is the
//! minimum a WPA2-PSK network with CCMP needs: PSK derivation (PBKDF2-SHA1), the PTK (PRF-384),
//! EAPOL-Key MICs (HMAC-SHA1-128, key descriptor version 2), and the GTK from AES key unwrap.
//!
//! Pure `core` and free of the kernel, so it is tested on the host (`tests` below, built with
//! `rustc --test`). The caller owns transport: it feeds received EAPOL frames to
//! `Supplicant::receive` and transmits what comes back, then installs the keys it reports.

#![cfg_attr(not(test), no_std)]

// ---- SHA-1 -----------------------------------------------------------------------------------

#[derive(Clone)]
pub struct Sha1 {
    h: [u32; 5],
    block: [u8; 64],
    fill: usize,
    total: u64,
}

impl Sha1 {
    pub fn new() -> Self {
        Sha1 { h: [0x6745_2301, 0xEFCD_AB89, 0x98BA_DCFE, 0x1032_5476, 0xC3D2_E1F0], block: [0; 64], fill: 0, total: 0 }
    }

    fn compress(&mut self) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([self.block[4 * i], self.block[4 * i + 1], self.block[4 * i + 2], self.block[4 * i + 3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let [mut a, mut b, mut c, mut d, mut e] = self.h;
        for (i, &wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | (!b & d), 0x5A82_7999),
                20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
                _ => (b ^ c ^ d, 0xCA62_C1D6),
            };
            let t = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = t;
        }
        for (h, v) in self.h.iter_mut().zip([a, b, c, d, e]) {
            *h = h.wrapping_add(v);
        }
    }

    pub fn update(&mut self, mut data: &[u8]) {
        self.total += data.len() as u64;
        while !data.is_empty() {
            let n = (64 - self.fill).min(data.len());
            self.block[self.fill..self.fill + n].copy_from_slice(&data[..n]);
            self.fill += n;
            data = &data[n..];
            if self.fill == 64 {
                self.compress();
                self.fill = 0;
            }
        }
    }

    pub fn digest(mut self) -> [u8; 20] {
        let bits = self.total * 8;
        self.update(&[0x80]);
        while self.fill != 56 {
            self.update(&[0]);
        }
        self.update(&bits.to_be_bytes());
        let mut out = [0u8; 20];
        for (i, h) in self.h.iter().enumerate() {
            out[4 * i..4 * i + 4].copy_from_slice(&h.to_be_bytes());
        }
        out
    }
}

/// HMAC-SHA1 keyed once and reused: the padded-key states are hashed up front, which is what
/// keeps PBKDF2's 8192 MACs cheap.
#[derive(Clone)]
pub struct HmacSha1 {
    inner: Sha1,
    outer: Sha1,
}

impl HmacSha1 {
    pub fn new(key: &[u8]) -> Self {
        let mut k = [0u8; 64];
        if key.len() > 64 {
            let mut s = Sha1::new();
            s.update(key);
            k[..20].copy_from_slice(&s.digest());
        } else {
            k[..key.len()].copy_from_slice(key);
        }
        let mut ipad = [0x36u8; 64];
        let mut opad = [0x5Cu8; 64];
        for i in 0..64 {
            ipad[i] ^= k[i];
            opad[i] ^= k[i];
        }
        let mut inner = Sha1::new();
        inner.update(&ipad);
        let mut outer = Sha1::new();
        outer.update(&opad);
        HmacSha1 { inner, outer }
    }

    pub fn mac(&self, parts: &[&[u8]]) -> [u8; 20] {
        let mut i = self.inner.clone();
        for p in parts {
            i.update(p);
        }
        let mut o = self.outer.clone();
        o.update(&i.digest());
        o.digest()
    }
}

/// The PSK of a WPA2-Personal network: PBKDF2-HMAC-SHA1(passphrase, SSID, 4096, 32).
pub fn psk(passphrase: &[u8], ssid: &[u8]) -> [u8; 32] {
    let prf = HmacSha1::new(passphrase);
    let mut out = [0u8; 32];
    for block in 1..=2u32 {
        let mut u = prf.mac(&[ssid, &block.to_be_bytes()]);
        let mut t = u;
        for _ in 1..4096 {
            u = prf.mac(&[&u]);
            for (x, y) in t.iter_mut().zip(u.iter()) {
                *x ^= y;
            }
        }
        let at = (block as usize - 1) * 20;
        let n = (32 - at).min(20);
        out[at..at + n].copy_from_slice(&t[..n]);
    }
    out
}

/// PRF-n (802.11-2016 12.7.1.2): HMAC-SHA1(K, A || 0 || B || i) for i = 0, 1, ...
pub fn prf(key: &[u8], label: &[u8], data: &[&[u8]], out: &mut [u8]) {
    let h = HmacSha1::new(key);
    let mut done = 0;
    let mut i = 0u8;
    while done < out.len() {
        let mut parts: [&[u8]; 8] = [&[]; 8];
        parts[0] = label;
        parts[1] = &[0];
        let n = data.len().min(5);
        parts[2..2 + n].copy_from_slice(&data[..n]);
        let counter = [i];
        parts[2 + n] = &counter;
        let block = h.mac(&parts[..3 + n]);
        let k = (out.len() - done).min(20);
        out[done..done + k].copy_from_slice(&block[..k]);
        done += k;
        i += 1;
    }
}

// ---- AES-128 (decryption only: key unwrap needs nothing else) --------------------------------

const SBOX: [u8; 256] = {
    // The S-box generated from its definition: the multiplicative inverse in GF(2^8), then
    // the affine transform. Built at compile time so no table is typed by hand.
    let mut s = [0u8; 256];
    let mut p: u8 = 1;
    let mut q: u8 = 1;
    loop {
        // p walks the group by multiplying by 3; q by dividing by 3, so q = p^-1.
        p = p ^ (p << 1) ^ (if p & 0x80 != 0 { 0x1B } else { 0 });
        q ^= q << 1;
        q ^= q << 2;
        q ^= q << 4;
        if q & 0x80 != 0 {
            q ^= 0x09;
        }
        let x = q ^ q.rotate_left(1) ^ q.rotate_left(2) ^ q.rotate_left(3) ^ q.rotate_left(4);
        s[p as usize] = x ^ 0x63;
        if p == 1 {
            break;
        }
    }
    s[0] = 0x63;
    s
};

const INV_SBOX: [u8; 256] = {
    let mut inv = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        inv[SBOX[i] as usize] = i as u8;
        i += 1;
    }
    inv
};

fn xtime(x: u8) -> u8 {
    (x << 1) ^ (if x & 0x80 != 0 { 0x1B } else { 0 })
}

fn gmul(mut a: u8, mut b: u8) -> u8 {
    let mut r = 0;
    while b != 0 {
        if b & 1 != 0 {
            r ^= a;
        }
        a = xtime(a);
        b >>= 1;
    }
    r
}

pub struct Aes128 {
    rk: [[u8; 16]; 11],
}

impl Aes128 {
    pub fn new(key: &[u8; 16]) -> Self {
        let mut rk = [[0u8; 16]; 11];
        rk[0] = *key;
        let mut rcon = 1u8;
        for r in 1..11 {
            let p = rk[r - 1];
            let mut t = [p[13], p[14], p[15], p[12]];
            for b in t.iter_mut() {
                *b = SBOX[*b as usize];
            }
            t[0] ^= rcon;
            rcon = xtime(rcon);
            let mut k = [0u8; 16];
            for i in 0..4 {
                k[i] = p[i] ^ t[i];
            }
            for i in 4..16 {
                k[i] = p[i] ^ k[i - 4];
            }
            rk[r] = k;
        }
        Aes128 { rk }
    }

    pub fn encrypt(&self, b: &mut [u8; 16]) {
        xor16(b, &self.rk[0]);
        for r in 1..11 {
            for x in b.iter_mut() {
                *x = SBOX[*x as usize];
            }
            shift_rows(b);
            if r != 10 {
                for c in 0..4 {
                    let col = [b[4 * c], b[4 * c + 1], b[4 * c + 2], b[4 * c + 3]];
                    for i in 0..4 {
                        b[4 * c + i] = gmul(col[i], 2) ^ gmul(col[(i + 1) % 4], 3) ^ col[(i + 2) % 4] ^ col[(i + 3) % 4];
                    }
                }
            }
            xor16(b, &self.rk[r]);
        }
    }

    pub fn decrypt(&self, b: &mut [u8; 16]) {
        xor16(b, &self.rk[10]);
        for r in (0..10).rev() {
            inv_shift_rows(b);
            for x in b.iter_mut() {
                *x = INV_SBOX[*x as usize];
            }
            xor16(b, &self.rk[r]);
            if r != 0 {
                for c in 0..4 {
                    let col = [b[4 * c], b[4 * c + 1], b[4 * c + 2], b[4 * c + 3]];
                    for i in 0..4 {
                        b[4 * c + i] = gmul(col[i], 14) ^ gmul(col[(i + 1) % 4], 11) ^ gmul(col[(i + 2) % 4], 13)
                            ^ gmul(col[(i + 3) % 4], 9);
                    }
                }
            }
        }
    }
}

fn xor16(b: &mut [u8; 16], k: &[u8; 16]) {
    for i in 0..16 {
        b[i] ^= k[i];
    }
}

// The state is column-major: byte 4c + r is row r of column c. Row r rotates left by r.
fn shift_rows(b: &mut [u8; 16]) {
    let s = *b;
    for c in 0..4 {
        for r in 0..4 {
            b[4 * c + r] = s[4 * ((c + r) % 4) + r];
        }
    }
}

fn inv_shift_rows(b: &mut [u8; 16]) {
    let s = *b;
    for c in 0..4 {
        for r in 0..4 {
            b[4 * ((c + r) % 4) + r] = s[4 * c + r];
        }
    }
}

/// AES key unwrap (RFC 3394) of `wrapped` (n + 1 64-bit blocks) into `out` (n blocks).
/// False when the integrity check fails, i.e. the wrong KEK.
pub fn aes_unwrap(kek: &[u8; 16], wrapped: &[u8], out: &mut [u8]) -> bool {
    if wrapped.len() < 24 || wrapped.len() % 8 != 0 || out.len() < wrapped.len() - 8 {
        return false;
    }
    let n = wrapped.len() / 8 - 1;
    let aes = Aes128::new(kek);
    let mut a = [0u8; 8];
    a.copy_from_slice(&wrapped[..8]);
    out[..8 * n].copy_from_slice(&wrapped[8..]);
    for j in (0..6).rev() {
        for i in (1..=n).rev() {
            let t = (n * j + i) as u64;
            let mut b = [0u8; 16];
            for (k, x) in t.to_be_bytes().iter().enumerate() {
                b[k] = a[k] ^ x;
            }
            b[8..].copy_from_slice(&out[8 * (i - 1)..8 * i]);
            aes.decrypt(&mut b);
            a.copy_from_slice(&b[..8]);
            out[8 * (i - 1)..8 * i].copy_from_slice(&b[8..]);
        }
    }
    a == [0xA6; 8]
}

// ---- EAPOL-Key -------------------------------------------------------------------------------

/// Key Information bits (802.11-2016 Figure 12-33).
const KI_VERSION_MASK: u16 = 0x0007;
const KI_VERSION_HMAC_SHA1_AES: u16 = 2;
const KI_PAIRWISE: u16 = 0x0008;
const KI_INSTALL: u16 = 0x0040;
const KI_MIC: u16 = 0x0100;
const KI_SECURE: u16 = 0x0200;
const KI_ENCRYPTED_DATA: u16 = 0x1000;

const EAPOL_KEY: u8 = 3;
const DESC_RSN: u8 = 2;

// Offsets into an EAPOL frame (from the 802.1X header) carrying an EAPOL-Key body.
const O_DESC: usize = 4;
const O_INFO: usize = 5;
const O_KEYLEN: usize = 7;
const O_REPLAY: usize = 9;
const O_NONCE: usize = 17;
const O_RSC: usize = 65;
const O_MIC: usize = 81;
const O_DATALEN: usize = 97;
const O_DATA: usize = 99;

/// The longest EAPOL frame built here: the header and an RSN element of up to 64 bytes.
pub const MAX_REPLY: usize = O_DATA + 64;

/// A group key, as the network's GTK KDE and the EAPOL-Key frame carried it.
#[derive(Clone, Copy)]
pub struct GroupKey {
    pub key: [u8; 32],
    pub len: usize,
    pub id: u8,
    /// Receive sequence counter: the low 6 bytes of Key RSC, PN0 first.
    pub rsc: [u8; 6],
}

/// What the caller does with a received frame's outcome.
pub struct Reply {
    /// The EAPOL frame to transmit (from the 802.1X header), when `len` is nonzero.
    pub frame: [u8; MAX_REPLY],
    pub len: usize,
    /// After transmitting: install the pairwise temporal key (TK, `Supplicant::tk`).
    pub install_ptk: bool,
    /// After transmitting: install this group key.
    pub gtk: Option<GroupKey>,
    /// The 4-way handshake finished with this frame.
    pub complete: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    NotEapolKey,
    Truncated,
    UnsupportedVersion,
    Replayed,
    NoPtk,
    BadMic,
    NonceChanged,
    UnwrapFailed,
    NoGtk,
}

pub struct Supplicant {
    pmk: [u8; 32],
    /// The authenticator's (AP's) and the supplicant's (our) MAC addresses.
    aa: [u8; 6],
    spa: [u8; 6],
    /// Our RSN element, byte for byte as the association request sent it: message 2 must
    /// repeat it or the AP drops the handshake.
    rsn: [u8; 64],
    rsn_len: usize,
    snonce: [u8; 32],
    anonce: [u8; 32],
    ptk: [u8; 48],
    have_ptk: bool,
    replay: [u8; 8],
    have_replay: bool,
}

impl Supplicant {
    /// `snonce` must be fresh randomness for this association.
    pub fn new(pmk: &[u8; 32], aa: &[u8; 6], spa: &[u8; 6], rsn: &[u8], snonce: &[u8; 32]) -> Self {
        let mut s = Supplicant {
            pmk: *pmk,
            aa: *aa,
            spa: *spa,
            rsn: [0; 64],
            rsn_len: rsn.len().min(64),
            snonce: *snonce,
            anonce: [0; 32],
            ptk: [0; 48],
            have_ptk: false,
            replay: [0; 8],
            have_replay: false,
        };
        s.rsn[..s.rsn_len].copy_from_slice(&rsn[..s.rsn_len]);
        s
    }

    /// The CCMP temporal key, once message 3 has been verified.
    pub fn tk(&self) -> &[u8] {
        &self.ptk[32..48]
    }

    fn kck(&self) -> &[u8] {
        &self.ptk[..16]
    }

    fn kek(&self) -> [u8; 16] {
        let mut k = [0u8; 16];
        k.copy_from_slice(&self.ptk[16..32]);
        k
    }

    fn derive_ptk(&mut self) {
        let (a1, a2) = if self.aa < self.spa { (self.aa, self.spa) } else { (self.spa, self.aa) };
        let (n1, n2) = if self.anonce < self.snonce { (self.anonce, self.snonce) } else { (self.snonce, self.anonce) };
        let mut ptk = [0u8; 48];
        prf(&self.pmk, b"Pairwise key expansion", &[&a1, &a2, &n1, &n2], &mut ptk);
        self.ptk = ptk;
        self.have_ptk = true;
    }

    fn mic(&self, frame: &[u8]) -> [u8; 16] {
        let h = HmacSha1::new(self.kck());
        let zeros = [0u8; 16];
        let full = h.mac(&[&frame[..O_MIC], &zeros, &frame[O_MIC + 16..]]);
        let mut m = [0u8; 16];
        m.copy_from_slice(&full[..16]);
        m
    }

    /// Process one received EAPOL frame (from the 802.1X header; the LLC/SNAP is gone).
    pub fn receive(&mut self, frame: &[u8], reply: &mut Reply) -> Result<(), Error> {
        reply.len = 0;
        reply.install_ptk = false;
        reply.gtk = None;
        reply.complete = false;
        if frame.len() < 4 || frame[1] != EAPOL_KEY {
            return Err(Error::NotEapolKey);
        }
        let body = u16::from_be_bytes([frame[2], frame[3]]) as usize;
        if frame.len() < 4 + body || body < O_DATA - 4 {
            return Err(Error::Truncated);
        }
        let frame = &frame[..4 + body];
        if frame[O_DESC] != DESC_RSN {
            return Err(Error::NotEapolKey);
        }
        let info = u16::from_be_bytes([frame[O_INFO], frame[O_INFO + 1]]);
        if info & KI_VERSION_MASK != KI_VERSION_HMAC_SHA1_AES {
            return Err(Error::UnsupportedVersion);
        }
        let data_len = u16::from_be_bytes([frame[O_DATALEN], frame[O_DATALEN + 1]]) as usize;
        if frame.len() < O_DATA + data_len {
            return Err(Error::Truncated);
        }
        let mut replay = [0u8; 8];
        replay.copy_from_slice(&frame[O_REPLAY..O_REPLAY + 8]);
        if self.have_replay && replay <= self.replay {
            return Err(Error::Replayed);
        }

        if info & KI_MIC != 0 {
            if !self.have_ptk {
                return Err(Error::NoPtk);
            }
            if self.mic(frame) != frame[O_MIC..O_MIC + 16] {
                return Err(Error::BadMic);
            }
        }

        let pairwise = info & KI_PAIRWISE != 0;
        if pairwise && info & KI_MIC == 0 {
            // Message 1: the AP's nonce. A retransmitted message 1 restarts the handshake
            // with the same SNonce, which the standard allows.
            self.anonce.copy_from_slice(&frame[O_NONCE..O_NONCE + 32]);
            self.derive_ptk();
            self.replay = replay;
            self.have_replay = true;
            let rsn_len = self.rsn_len;
            let rsn = self.rsn;
            let snonce = self.snonce;
            self.build(frame[0], KI_VERSION_HMAC_SHA1_AES | KI_PAIRWISE | KI_MIC, &replay, &snonce, &rsn[..rsn_len], reply);
            return Ok(());
        }

        // Message 3 or a group message 1: both carry the GTK in encrypted key data.
        if pairwise && frame[O_NONCE..O_NONCE + 32] != self.anonce {
            return Err(Error::NonceChanged);
        }
        let gtk = if info & KI_ENCRYPTED_DATA != 0 {
            let mut plain = [0u8; 256];
            let wrapped = &frame[O_DATA..O_DATA + data_len];
            if wrapped.len() > plain.len() + 8 || !aes_unwrap(&self.kek(), wrapped, &mut plain) {
                return Err(Error::UnwrapFailed);
            }
            let mut rsc = [0u8; 6];
            rsc.copy_from_slice(&frame[O_RSC..O_RSC + 6]);
            find_gtk(&plain[..wrapped.len() - 8], rsc)
        } else {
            None
        };
        if gtk.is_none() && (!pairwise || info & KI_INSTALL != 0) {
            return Err(Error::NoGtk);
        }
        self.replay = replay;
        self.have_replay = true;
        if pairwise {
            // Message 4: no nonce, no key data.
            self.build(frame[0], KI_VERSION_HMAC_SHA1_AES | KI_PAIRWISE | KI_MIC | KI_SECURE, &replay, &[0; 32], &[], reply);
            reply.install_ptk = info & KI_INSTALL != 0;
            reply.complete = true;
        } else {
            self.build(frame[0], KI_VERSION_HMAC_SHA1_AES | KI_MIC | KI_SECURE, &replay, &[0; 32], &[], reply);
        }
        reply.gtk = gtk;
        Ok(())
    }

    fn build(&self, version: u8, info: u16, replay: &[u8; 8], nonce: &[u8; 32], data: &[u8], reply: &mut Reply) {
        let len = O_DATA + data.len();
        let f = &mut reply.frame[..len];
        f.fill(0);
        f[0] = version;
        f[1] = EAPOL_KEY;
        f[2..4].copy_from_slice(&((len - 4) as u16).to_be_bytes());
        f[O_DESC] = DESC_RSN;
        f[O_INFO..O_INFO + 2].copy_from_slice(&info.to_be_bytes());
        f[O_KEYLEN..O_KEYLEN + 2].copy_from_slice(&0u16.to_be_bytes());
        f[O_REPLAY..O_REPLAY + 8].copy_from_slice(replay);
        f[O_NONCE..O_NONCE + 32].copy_from_slice(nonce);
        f[O_DATALEN..O_DATALEN + 2].copy_from_slice(&(data.len() as u16).to_be_bytes());
        f[O_DATA..].copy_from_slice(data);
        let mic = self.mic(f);
        f[O_MIC..O_MIC + 16].copy_from_slice(&mic);
        reply.len = len;
    }
}

/// The GTK KDE (dd, len, 00-0F-AC, 1, then key ID/Tx, reserved, GTK) in decrypted key data.
fn find_gtk(kd: &[u8], rsc: [u8; 6]) -> Option<GroupKey> {
    let mut i = 0;
    while i + 2 <= kd.len() {
        let (id, len) = (kd[i], kd[i + 1] as usize);
        if id == 0xDD && len == 0 {
            break; // padding
        }
        if i + 2 + len > kd.len() {
            break;
        }
        let e = &kd[i + 2..i + 2 + len];
        if id == 0xDD && len >= 6 + 16 && e[..4] == [0x00, 0x0F, 0xAC, 0x01] {
            let k = &e[6..];
            let n = k.len().min(32);
            let mut g = GroupKey { key: [0; 32], len: n, id: e[4] & 3, rsc };
            g.key[..n].copy_from_slice(&k[..n]);
            return Some(g);
        }
        i += 2 + len;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(s: &str) -> std::vec::Vec<u8> {
        let s: std::string::String = s.chars().filter(|c| !c.is_whitespace()).collect();
        (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
    }

    #[test]
    fn sha1_vectors() {
        let mut s = Sha1::new();
        s.update(b"abc");
        assert_eq!(s.digest().to_vec(), hex("a9993e364706816aba3e25717850c26c9cd0d89d"));
        let mut s = Sha1::new();
        s.update(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");
        assert_eq!(s.digest().to_vec(), hex("84983e441c3bd26ebaae4aa1f95129e5e54670f1"));
    }

    #[test]
    fn hmac_vector() {
        // RFC 2202 test case 2.
        let m = HmacSha1::new(b"Jefe").mac(&[b"what do ya want ", b"for nothing?"]);
        assert_eq!(m.to_vec(), hex("effcdf6ae5eb2fa2d27416d5f184df9c259a7c79"));
    }

    #[test]
    fn psk_vector() {
        // IEEE 802.11-2016 J.4.2.
        assert_eq!(psk(b"password", b"IEEE").to_vec(),
            hex("f42c6fc52df0ebef9ebb4b90b38a5f902e83fe1b135a70e23aed762e9710a12e"));
        assert_eq!(psk(b"ThisIsAPassword", b"ThisIsASSID").to_vec(),
            hex("0dc0d6eb90555ed6419756b9a15ec3e3209b63df707dd508d14581f8982721af"));
    }

    #[test]
    fn prf_vector() {
        // IEEE 802.11-2016 J.3.2, test case 1.
        let mut out = [0u8; 64];
        prf(&[0x0B; 20], b"prefix", &[b"Hi There"], &mut out);
        assert_eq!(out.to_vec(), hex("bcd4c650b30b9684951829e0d75f9d54b862175ed9f00606e17d8da35402ffee
            75df78c3d31e0f889f012120c0862beb67753e7439ae242edb8373698356cf5a"));
    }

    #[test]
    fn aes_vector() {
        // FIPS-197 C.1.
        let key: [u8; 16] = hex("000102030405060708090a0b0c0d0e0f").try_into().unwrap();
        let mut b: [u8; 16] = hex("00112233445566778899aabbccddeeff").try_into().unwrap();
        let aes = Aes128::new(&key);
        aes.encrypt(&mut b);
        assert_eq!(b.to_vec(), hex("69c4e0d86a7b0430d8cdb78070b4c55a"));
        aes.decrypt(&mut b);
        assert_eq!(b.to_vec(), hex("00112233445566778899aabbccddeeff"));
    }

    #[test]
    fn unwrap_vector() {
        // RFC 3394 4.1.
        let kek: [u8; 16] = hex("000102030405060708090a0b0c0d0e0f").try_into().unwrap();
        let mut out = [0u8; 16];
        assert!(aes_unwrap(&kek, &hex("1fa68b0a8112b447aef34bd8fb5a7b829d3e862371d2cfe5"), &mut out));
        assert_eq!(out.to_vec(), hex("00112233445566778899aabbccddeeff"));
        assert!(!aes_unwrap(&[1; 16], &hex("1fa68b0a8112b447aef34bd8fb5a7b829d3e862371d2cfe5"), &mut out));
    }

    /// The authenticator's side, enough to drive a handshake against `Supplicant`.
    fn ap_frame(info: u16, replay: u64, nonce: &[u8; 32], rsc: &[u8; 8], data: &[u8], kck: Option<&[u8]>) -> std::vec::Vec<u8> {
        let mut f = std::vec![0u8; O_DATA + data.len()];
        f[0] = 2;
        f[1] = EAPOL_KEY;
        let n = (f.len() - 4) as u16;
        f[2..4].copy_from_slice(&n.to_be_bytes());
        f[O_DESC] = DESC_RSN;
        f[O_INFO..O_INFO + 2].copy_from_slice(&info.to_be_bytes());
        f[O_KEYLEN..O_KEYLEN + 2].copy_from_slice(&16u16.to_be_bytes());
        f[O_REPLAY..O_REPLAY + 8].copy_from_slice(&replay.to_be_bytes());
        f[O_NONCE..O_NONCE + 32].copy_from_slice(nonce);
        f[O_RSC..O_RSC + 8].copy_from_slice(rsc);
        f[O_DATALEN..O_DATALEN + 2].copy_from_slice(&(data.len() as u16).to_be_bytes());
        f[O_DATA..].copy_from_slice(data);
        if let Some(kck) = kck {
            let m = HmacSha1::new(kck).mac(&[&f]);
            f[O_MIC..O_MIC + 16].copy_from_slice(&m[..16]);
        }
        f
    }

    /// AES key wrap (RFC 3394), for the test authenticator.
    fn wrap(kek: &[u8; 16], plain: &[u8]) -> std::vec::Vec<u8> {
        let n = plain.len() / 8;
        let aes = Aes128::new(kek);
        let mut a = [0xA6u8; 8];
        let mut r = plain.to_vec();
        for j in 0..6 {
            for i in 1..=n {
                let mut b = [0u8; 16];
                b[..8].copy_from_slice(&a);
                b[8..].copy_from_slice(&r[8 * (i - 1)..8 * i]);
                aes.encrypt(&mut b);
                let t = ((n * j) + i) as u64;
                for (k, x) in t.to_be_bytes().iter().enumerate() {
                    a[k] = b[k] ^ x;
                }
                r[8 * (i - 1)..8 * i].copy_from_slice(&b[8..]);
            }
        }
        let mut out = a.to_vec();
        out.extend_from_slice(&r);
        out
    }

    #[test]
    fn four_way_handshake() {
        let pmk = psk(b"correct horse", b"Piconet");
        let aa = [0x10, 0x7c, 0x61, 0xa7, 0x7b, 0x21];
        let spa = [0x90, 0xde, 0x80, 0xd6, 0x50, 0x6f];
        let rsn = hex("30140100000fac040100000fac040100000fac020000");
        let snonce = [0x5A; 32];
        let anonce = [0xA5; 32];
        let mut s = Supplicant::new(&pmk, &aa, &spa, &rsn, &snonce);
        let mut reply = Reply { frame: [0; MAX_REPLY], len: 0, install_ptk: false, gtk: None, complete: false };

        // Message 1.
        let m1 = ap_frame(0x008A, 1, &anonce, &[0; 8], &[], None);
        s.receive(&m1, &mut reply).unwrap();
        let m2 = reply.frame[..reply.len].to_vec();
        assert_eq!(u16::from_be_bytes([m2[O_INFO], m2[O_INFO + 1]]), 0x010A);
        assert_eq!(&m2[O_NONCE..O_NONCE + 32], &snonce);
        assert_eq!(&m2[O_DATA..], &rsn[..]);

        // The authenticator derives the same PTK and checks message 2's MIC.
        // Addresses and nonces go in ascending order: here SNonce sorts first.
        let mut ptk = [0u8; 48];
        prf(&pmk, b"Pairwise key expansion", &[&aa, &spa, &snonce, &anonce], &mut ptk);
        let (kck, kek) = (&ptk[..16], <[u8; 16]>::try_from(&ptk[16..32]).unwrap());
        let mut z = m2.clone();
        z[O_MIC..O_MIC + 16].fill(0);
        assert_eq!(&HmacSha1::new(kck).mac(&[&z])[..16], &m2[O_MIC..O_MIC + 16]);

        // Message 3: our RSN element and a GTK KDE, wrapped, padded to a multiple of 8.
        let gtk = [0x47u8; 16];
        let mut kd = rsn.clone();
        kd.extend_from_slice(&[0xDD, 22, 0x00, 0x0F, 0xAC, 0x01, 0x01, 0x00]);
        kd.extend_from_slice(&gtk);
        kd.push(0xDD);
        while kd.len() % 8 != 0 {
            kd.push(0);
        }
        let m3 = ap_frame(0x13CA, 2, &anonce, &[7, 0, 0, 0, 0, 0, 0, 0], &wrap(&kek, &kd), Some(kck));
        s.receive(&m3, &mut reply).unwrap();
        assert!(reply.install_ptk && reply.complete);
        let g = reply.gtk.unwrap();
        assert_eq!((&g.key[..g.len], g.id, g.rsc[0]), (&gtk[..], 1, 7));
        assert_eq!(s.tk(), &ptk[32..48]);
        let m4 = reply.frame[..reply.len].to_vec();
        assert_eq!(u16::from_be_bytes([m4[O_INFO], m4[O_INFO + 1]]), 0x030A);
        let mut z = m4.clone();
        z[O_MIC..O_MIC + 16].fill(0);
        assert_eq!(&HmacSha1::new(kck).mac(&[&z])[..16], &m4[O_MIC..O_MIC + 16]);

        // A replayed message 3 is refused; a tampered one fails its MIC.
        assert_eq!(s.receive(&m3, &mut reply).err(), Some(Error::Replayed));
        let mut bad = ap_frame(0x13CA, 3, &anonce, &[0; 8], &wrap(&kek, &kd), Some(kck));
        bad[O_DATA] ^= 1;
        assert_eq!(s.receive(&bad, &mut reply).err(), Some(Error::BadMic));

        // Group rekey: message 1 of the group handshake, answered with message 2.
        let gtk2 = [0x99u8; 16];
        let mut kd = std::vec![0xDD, 22, 0x00, 0x0F, 0xAC, 0x01, 0x02, 0x00];
        kd.extend_from_slice(&gtk2);
        while kd.len() % 8 != 0 {
            kd.push(0xDD);
            while kd.len() % 8 != 0 {
                kd.push(0);
            }
        }
        let g1 = ap_frame(0x1382, 4, &[0; 32], &[0; 8], &wrap(&kek, &kd), Some(kck));
        s.receive(&g1, &mut reply).unwrap();
        let g = reply.gtk.unwrap();
        assert_eq!((&g.key[..g.len], g.id), (&gtk2[..], 2));
        let g2 = &reply.frame[..reply.len];
        assert_eq!(u16::from_be_bytes([g2[O_INFO], g2[O_INFO + 1]]), 0x0302);
        assert!(!reply.install_ptk && !reply.complete);
    }
}
