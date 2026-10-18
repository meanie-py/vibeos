//! The SNTP packet (RFC 4330): the client's request, the checks a reply must pass,
//! and NTP time to NT time. No kernel or smoltcp dependency, so the tests run on the
//! host: `rustc --edition 2021 --test wire.rs && ./wire`.

pub const PORT: u16 = 123;
pub const LEN: usize = 48;
/// Seconds from 1601-01-01 (NT's epoch) to 1900-01-01 (NTP era 0).
const NT_TO_NTP_SECS: u64 = 9_435_484_800;

/// A client request. The server copies its transmit timestamp, `nonce`, into the
/// reply's originate timestamp, which is how a reply is matched to the request.
pub fn request(nonce: u64) -> [u8; LEN] {
    let mut p = [0u8; LEN];
    p[0] = 0x23; // LI 0, version 4, mode 3 (client)
    p[40..48].copy_from_slice(&nonce.to_be_bytes());
    p
}

/// Why a reply was not used.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reject {
    Short,
    NotServer,
    /// Stratum 0: the server says go away (rate limit, access denied).
    KissOfDeath,
    /// Leap indicator 3 or stratum 16+: the server has no time to give.
    Unsynchronized,
    /// The originate timestamp is not our nonce: a stale or forged reply.
    NotOurs,
    NoTime,
}

/// The server's transmit time, in NT time (100 ns since 1601-01-01 UTC), from a
/// reply to `request(nonce)`.
pub fn reply(p: &[u8], nonce: u64) -> Result<u64, Reject> {
    if p.len() < LEN {
        return Err(Reject::Short);
    }
    if p[0] & 7 != 4 {
        return Err(Reject::NotServer);
    }
    if p[1] == 0 {
        return Err(Reject::KissOfDeath);
    }
    if p[0] >> 6 == 3 || p[1] >= 16 {
        return Err(Reject::Unsynchronized);
    }
    if p[24..32] != nonce.to_be_bytes() {
        return Err(Reject::NotOurs);
    }
    let secs = u32::from_be_bytes([p[40], p[41], p[42], p[43]]);
    let frac = u32::from_be_bytes([p[44], p[45], p[46], p[47]]);
    if secs == 0 && frac == 0 {
        return Err(Reject::NoTime);
    }
    Ok(nt(secs, frac))
}

/// NTP's seconds wrap in February 2036; a reading with the top bit clear is taken
/// as era 1, which covers 1968 to 2104 (RFC 4330 section 3).
pub fn nt(secs: u32, frac: u32) -> u64 {
    let era = if secs & 0x8000_0000 == 0 { 1u64 << 32 } else { 0 };
    (secs as u64 + era + NT_TO_NTP_SECS) * 10_000_000 + ((frac as u64 * 10_000_000) >> 32)
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNIX_TO_NTP: u64 = 2_208_988_800;
    const NT_TO_UNIX: u64 = 11_644_473_600;

    fn served(nonce: u64, secs: u32, frac: u32) -> [u8; LEN] {
        let mut p = [0u8; LEN];
        p[0] = 0x24; // LI 0, version 4, mode 4 (server)
        p[1] = 2;
        p[24..32].copy_from_slice(&nonce.to_be_bytes());
        p[40..44].copy_from_slice(&secs.to_be_bytes());
        p[44..48].copy_from_slice(&frac.to_be_bytes());
        p
    }

    #[test]
    fn the_request_is_a_v4_client_packet_carrying_the_nonce() {
        let p = request(0x0102_0304_0506_0708);
        assert_eq!(p[0], 0x23);
        assert_eq!(p[40..48], [1, 2, 3, 4, 5, 6, 7, 8]);
        assert!(p[1..40].iter().all(|&b| b == 0));
    }

    #[test]
    fn a_reply_today_converts_to_nt_time() {
        // 2026-10-18 00:00:00.5 UTC
        let unix = 1_792_281_600u64;
        let t = reply(&served(9, (unix + UNIX_TO_NTP) as u32, 0x8000_0000), 9).unwrap();
        assert_eq!(t, (unix + NT_TO_UNIX) * 10_000_000 + 5_000_000);
    }

    #[test]
    fn after_the_2036_wrap_the_time_keeps_counting() {
        // 2040-01-01 00:00:00 UTC is past 2^32 NTP seconds.
        let unix = 2_208_988_800u64;
        let ntp = unix + UNIX_TO_NTP;
        assert!(ntp > u32::MAX as u64);
        assert_eq!(nt(ntp as u32, 0), (unix + NT_TO_UNIX) * 10_000_000);
    }

    #[test]
    fn replies_that_carry_no_usable_time_are_refused() {
        let ok = served(7, 0xEB00_0000, 0);
        assert!(reply(&ok, 7).is_ok());
        assert_eq!(reply(&ok[..47], 7), Err(Reject::Short));
        assert_eq!(reply(&ok, 8), Err(Reject::NotOurs));
        let mut p = ok;
        p[0] = 0x23;
        assert_eq!(reply(&p, 7), Err(Reject::NotServer));
        let mut p = ok;
        p[1] = 0;
        assert_eq!(reply(&p, 7), Err(Reject::KissOfDeath));
        let mut p = ok;
        p[0] |= 0xC0;
        assert_eq!(reply(&p, 7), Err(Reject::Unsynchronized));
        let mut p = ok;
        p[1] = 16;
        assert_eq!(reply(&p, 7), Err(Reject::Unsynchronized));
        assert_eq!(reply(&served(7, 0, 0), 7), Err(Reject::NoTime));
    }
}
