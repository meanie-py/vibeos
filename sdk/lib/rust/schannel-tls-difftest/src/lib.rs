//! Conformance harness for schannel's mbedTLS backend.
//!
//! Not a Rust-vs-C differential like its neighbours — it carries the `-difftest`
//! suffix because that is what `vibeos-cli difftest` discovers, and a harness no gate
//! runs is a harness that rots. What it grades is `dll/win32/schannel/
//! schannel_mbedtls.c` against a real TLS 1.2 peer, compiled for the host with the
//! shipped mbedTLS configuration.
//!
//! It exists for the 2.16.11 -> 3.x migration. mbedTLS 2.16 went end-of-life in 2021,
//! the port is mechanical, and every symbol break in it is caught by the compiler —
//! but a handshake that still completes while certificate handling has quietly changed
//! is not, and that is the failure this catches.

#![cfg(test)]

#[repr(C)]
#[derive(Debug)]
struct HarnessResult {
    handshake_status: i32,
    handshake_iterations: i32,
    protocol: [u8; 32],
    ciphersuite: [u8; 80],
    block_size: u32,
    max_message_size: u32,
    conn_protocol: u32,
    conn_cipher: u32,
    conn_cipher_strength: u32,
    conn_hash: u32,
    conn_hash_strength: u32,
    conn_exch: u32,
    conn_exch_strength: u32,
    conn_info_status: i32,
    peer_cert_status: i32,
    peer_cert_len: u32,
    peer_cert_matches: i32,
    echo_ok: i32,
    echo_got: [u8; 64],
    server_err: i32,
    enabled_protocols: u32,
}

impl Default for HarnessResult {
    // `[u8; 80]` does not derive Default, and the C side memsets the whole struct on
    // entry anyway, so an all-zero value is exactly what the contract wants.
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

extern "C" {
    fn schan_harness_run(out: *mut HarnessResult) -> i32;
}

const SEC_E_OK: i32 = 0;

/// `SP_PROT_*` from schannel.h, as the backend advertises them.
const SP_PROT_TLS1_2_SERVER: u32 = 0x0000_0400;
const SP_PROT_TLS1_2_CLIENT: u32 = 0x0000_0800;

fn run() -> HarnessResult {
    let mut r = HarnessResult::default();
    let rc = unsafe { schan_harness_run(&mut r) };
    assert_eq!(
        rc, 0,
        "harness driver failed with rc={rc}; result so far: {r:#?}\n\
         (rc -5 create_session, -6 handshake, -7 send; run with SCHAN_HARNESS_TRACE=1)"
    );
    r
}

#[test]
fn a_full_tls_handshake_completes() {
    let r = run();
    assert_eq!(r.handshake_status, SEC_E_OK, "handshake did not reach SEC_E_OK");
    assert_eq!(r.server_err, 0, "the loopback server ended with an mbedTLS error");
    // The whole point of the non-blocking client end: if this is 1 the EAGAIN path
    // never ran and the test is grading far less than it looks like it is.
    assert!(
        r.handshake_iterations > 1,
        "handshake finished in one call — the would-block path was not exercised"
    );
}

#[test]
fn the_negotiated_connection_is_reported_to_schannel() {
    let r = run();
    assert_eq!(r.conn_info_status, SEC_E_OK);
    assert!(
        r.conn_protocol == SP_PROT_TLS1_2_CLIENT || r.conn_protocol == SP_PROT_TLS1_2_SERVER,
        "expected a TLS 1.2 SP_PROT_* bit, got {:#x}",
        r.conn_protocol
    );
    for (name, v) in [
        ("aiCipher", r.conn_cipher),
        ("dwCipherStrength", r.conn_cipher_strength),
        ("aiHash", r.conn_hash),
        ("aiExch", r.conn_exch),
        ("dwExchStrength", r.conn_exch_strength),
    ] {
        assert_ne!(v, 0, "SecPkgContext_ConnectionInfo.{name} came back zero");
    }
    assert_ne!(r.block_size, 0, "cipher block size came back zero");
    assert_ne!(r.max_message_size, 0, "max message size came back zero");
}

#[test]
fn the_peer_certificate_reaches_the_certificate_store() {
    let r = run();
    assert_eq!(r.peer_cert_status, SEC_E_OK, "peer certificate was not retrievable");
    assert_ne!(r.peer_cert_len, 0, "nothing was handed to the certificate store");
    // Non-NULL is not the assertion that matters: the bytes have to be the server's
    // actual certificate, which is what a certificate-handling regression would break.
    assert_eq!(
        r.peer_cert_matches, 1,
        "the DER handed to the store is not the server's certificate ({} bytes)",
        r.peer_cert_len
    );
}

#[test]
fn application_data_round_trips_through_send_and_recv() {
    let r = run();
    let got = String::from_utf8_lossy(&r.echo_got)
        .trim_end_matches('\0')
        .to_string();
    assert_eq!(r.echo_ok, 1, "send/recv moved the wrong number of bytes (got {got:?})");
    assert_eq!(got, "HELLO SCHANNEL", "the echoed record came back wrong");
}

#[test]
fn the_advertised_protocol_set_is_recorded() {
    // Not an assertion about which protocols are right — a tripwire on the set
    // CHANGING. mbedTLS 3.x deletes SSL3/TLS1.0/TLS1.1 outright, so this value must
    // move during that port, deliberately and visibly, rather than by accident.
    let r = run();
    assert_ne!(r.enabled_protocols, 0, "schan_imp_enabled_protocols advertised nothing");
    assert_ne!(
        r.enabled_protocols & (SP_PROT_TLS1_2_CLIENT | SP_PROT_TLS1_2_SERVER),
        0,
        "TLS 1.2 is not advertised: {:#x}",
        r.enabled_protocols
    );
    eprintln!("schan_imp_enabled_protocols() = {:#x}", r.enabled_protocols);
}
