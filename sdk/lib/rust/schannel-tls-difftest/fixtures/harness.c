/* Drives dll/win32/schannel/schannel_mbedtls.c through a real TLS 1.2 handshake.
 *
 * There is no TLS test anywhere in this tree, which is why an mbedTLS bump has been
 * put off for five years: the library is EOL, the port is mechanical, and nothing
 * could tell you afterwards whether the handshake still worked. This is that thing.
 *
 * Shape: a socketpair, a plain-mbedTLS server on one end in a thread, and the genuine
 * schan_imp_* backend on the other. The backend talks to its socket only through
 * schan_pull/schan_push, exactly as it does inside schannel.dll, so nothing about the
 * code under test is stubbed -- only the Win32 surface around it.
 *
 * The client end is NON-BLOCKING on purpose. schan_pull returning EAGAIN is what
 * drives the backend's ROS_SCHAN_IS_BLOCKING marshalling and its SEC_I_CONTINUE_NEEDED
 * loop, and that path carries more of this file's own logic than the handshake does.
 */
#include "precomp.h"
#include "harness_cert.h"

#include <mbedtls/ssl.h>
#include <mbedtls/net_sockets.h>
#include <mbedtls/entropy.h>
#include <mbedtls/ctr_drbg.h>
#include <mbedtls/x509_crt.h>
#include <mbedtls/pk.h>

#include <pthread.h>
#include <sys/socket.h>
#include <unistd.h>
#include <fcntl.h>
#include <poll.h>
#include <stdio.h>

/* ---------- Win32 stand-ins ---------- */

int schan_harness_tracing(void)
{
    static int on = -1;
    if (on < 0) on = getenv("SCHAN_HARNESS_TRACE") != NULL;
    return on;
}

HANDLE WINAPI GetProcessHeap(void) { return (HANDLE)(size_t)1; }

PVOID WINAPI HeapAlloc(HANDLE heap, DWORD flags, SIZE_T size)
{
    void *p = malloc(size ? size : 1);
    (void)heap;
    if (p && (flags & HEAP_ZERO_MEMORY)) memset(p, 0, size);
    return p;
}

BOOL WINAPI HeapFree(HANDLE heap, DWORD flags, PVOID p)
{
    (void)heap; (void)flags;
    free(p);
    return TRUE;
}

/* The backend hands the peer's DER to the certificate store and keeps the context it
 * gets back. We record the blob so the test can prove it is the server's actual
 * certificate rather than merely non-NULL. */
static unsigned char g_peer_der[4096];
static unsigned int  g_peer_der_len;

BOOL WINAPI CertAddEncodedCertificateToStore(HCERTSTORE store, DWORD enc, const BYTE *der,
                                             DWORD len, DWORD disp, PCCERT_CONTEXT *out)
{
    (void)store; (void)enc; (void)disp;
    if (len > sizeof(g_peer_der)) return FALSE;
    memcpy(g_peer_der, der, len);
    g_peer_der_len = len;
    if (out) *out = (PCCERT_CONTEXT)(size_t)1;
    return TRUE;
}

BOOL WINAPI CertFreeCertificateContext(PCCERT_CONTEXT ctx)
{
    (void)ctx;
    return TRUE;
}

DWORD WINAPI GetLastError(void) { return 0; }

/* ---------- transport ---------- */

/* schan_transport's first member is a `struct schan_context *`, an incomplete type in
 * the contract header. The harness parks its own connection there rather than
 * shadowing the struct, so schannel_priv.h stays the single definition. */
struct harness_conn { int fd; };

int schan_pull(struct schan_transport *t, void *buff, size_t *buff_len)
{
    struct harness_conn *c = (struct harness_conn *)t->ctx;
    ssize_t n = recv(c->fd, buff, *buff_len, 0);
    if (n < 0)
    {
        *buff_len = 0;
        return (errno == EAGAIN || errno == EWOULDBLOCK) ? EAGAIN : errno;
    }
    *buff_len = (size_t)n;
    return NO_ERROR;
}

int schan_push(struct schan_transport *t, const void *buff, size_t *buff_len)
{
    struct harness_conn *c = (struct harness_conn *)t->ctx;
    ssize_t n = send(c->fd, buff, *buff_len, 0);
    if (n < 0)
    {
        *buff_len = 0;
        return (errno == EAGAIN || errno == EWOULDBLOCK) ? EAGAIN : errno;
    }
    *buff_len = (size_t)n;
    return NO_ERROR;
}

schan_imp_session schan_session_for_transport(struct schan_transport *t)
{
    (void)t;
    return NULL;
}

/* ---------- the loopback server ---------- */

struct server_args
{
    int fd;
    int err;
};

static int srv_send(void *ctx, const unsigned char *buf, size_t len)
{
    ssize_t n = send(*(int *)ctx, buf, len, 0);
    return n < 0 ? MBEDTLS_ERR_NET_SEND_FAILED : (int)n;
}

static int srv_recv(void *ctx, unsigned char *buf, size_t len)
{
    ssize_t n = recv(*(int *)ctx, buf, len, 0);
    return n < 0 ? MBEDTLS_ERR_NET_RECV_FAILED : (int)n;
}

static void *server_thread(void *p)
{
    struct server_args *a = p;
    mbedtls_ssl_context ssl;
    mbedtls_ssl_config conf;
    mbedtls_entropy_context entropy;
    mbedtls_ctr_drbg_context drbg;
    mbedtls_x509_crt crt;
    mbedtls_pk_context key;
    unsigned char buf[512];
    int ret;

    mbedtls_ssl_init(&ssl);
    mbedtls_ssl_config_init(&conf);
    mbedtls_entropy_init(&entropy);
    mbedtls_ctr_drbg_init(&drbg);
    mbedtls_x509_crt_init(&crt);
    mbedtls_pk_init(&key);

    if ((ret = mbedtls_ctr_drbg_seed(&drbg, mbedtls_entropy_func, &entropy,
                                     (const unsigned char *)"schanharness", 12)) != 0) goto out;
    if ((ret = mbedtls_x509_crt_parse_der(&crt, schan_harness_crt_der,
                                          schan_harness_crt_der_len)) != 0) goto out;
    if ((ret = mbedtls_pk_parse_key(&key, schan_harness_key_der,
                                    schan_harness_key_der_len, NULL, 0)) != 0) goto out;
    if ((ret = mbedtls_ssl_config_defaults(&conf, MBEDTLS_SSL_IS_SERVER,
                                           MBEDTLS_SSL_TRANSPORT_STREAM,
                                           MBEDTLS_SSL_PRESET_DEFAULT)) != 0) goto out;
    mbedtls_ssl_conf_rng(&conf, mbedtls_ctr_drbg_random, &drbg);
    if ((ret = mbedtls_ssl_conf_own_cert(&conf, &crt, &key)) != 0) goto out;
    if ((ret = mbedtls_ssl_setup(&ssl, &conf)) != 0) goto out;
    mbedtls_ssl_set_bio(&ssl, &a->fd, srv_send, srv_recv, NULL);

    while ((ret = mbedtls_ssl_handshake(&ssl)) != 0)
        if (ret != MBEDTLS_ERR_SSL_WANT_READ && ret != MBEDTLS_ERR_SSL_WANT_WRITE) goto out;

    /* Echo one record back uppercased, so the test can tell a real round-trip from a
     * handshake that completed and then moved no application data. */
    ret = mbedtls_ssl_read(&ssl, buf, sizeof(buf) - 1);
    if (ret <= 0) goto out;
    for (int i = 0; i < ret; i++)
        if (buf[i] >= 'a' && buf[i] <= 'z') buf[i] = (unsigned char)(buf[i] - 'a' + 'A');
    {
        int len = ret;
        ret = mbedtls_ssl_write(&ssl, buf, (size_t)len);
        if (ret != len) goto out;
    }
    mbedtls_ssl_close_notify(&ssl);
    ret = 0;

out:
    a->err = ret;
    mbedtls_pk_free(&key);
    mbedtls_x509_crt_free(&crt);
    mbedtls_ctr_drbg_free(&drbg);
    mbedtls_entropy_free(&entropy);
    mbedtls_ssl_config_free(&conf);
    mbedtls_ssl_free(&ssl);
    return NULL;
}

/* ---------- the exported driver ---------- */

typedef struct
{
    int      handshake_status;
    int      handshake_iterations;
    char     protocol[32];
    char     ciphersuite[80];
    unsigned block_size;
    unsigned max_message_size;
    unsigned conn_protocol;
    unsigned conn_cipher;
    unsigned conn_cipher_strength;
    unsigned conn_hash;
    unsigned conn_hash_strength;
    unsigned conn_exch;
    unsigned conn_exch_strength;
    int      conn_info_status;
    int      peer_cert_status;
    unsigned peer_cert_len;
    int      peer_cert_matches;
    int      echo_ok;
    char     echo_got[64];
    int      server_err;
    unsigned enabled_protocols;
} schan_harness_result;

#define MAX_HANDSHAKE_STEPS 512
#define STEP_WAIT_MS        5000

/* SEC_I_CONTINUE_NEEDED means "no more input right now", which is a request for the
 * caller to bring more -- not an invitation to spin. Busy-looping here burned all 512
 * steps in microseconds while the server was still doing its RSA work, which looks
 * exactly like a broken handshake and is not one. Waiting on the socket is also what
 * schannel's real caller does. */
static int wait_readable(int fd)
{
    struct pollfd pfd = { .fd = fd, .events = POLLIN };
    return poll(&pfd, 1, STEP_WAIT_MS);
}

int schan_harness_run(schan_harness_result *r)
{
    int sv[2];
    pthread_t th;
    struct server_args sa;
    struct harness_conn conn;
    struct schan_transport transport;
    struct schan_credentials cred;
    schan_imp_session session = NULL;
    SecPkgContext_ConnectionInfo info;
    PCCERT_CONTEXT cert = NULL;
    SECURITY_STATUS st;
    const char *msg = "hello schannel";
    SIZE_T len;
    char got[64];
    int rc = -1;

    memset(r, 0, sizeof(*r));
    g_peer_der_len = 0;

    if (!schan_imp_init()) return -2;
    r->enabled_protocols = (unsigned)schan_imp_enabled_protocols();

    if (socketpair(AF_UNIX, SOCK_STREAM, 0, sv) != 0) return -3;
    sa.fd = sv[1];
    sa.err = 0;
    if (pthread_create(&th, NULL, server_thread, &sa) != 0) { close(sv[0]); close(sv[1]); return -4; }

    fcntl(sv[0], F_SETFL, fcntl(sv[0], F_GETFL, 0) | O_NONBLOCK);
    conn.fd = sv[0];
    memset(&transport, 0, sizeof(transport));
    transport.ctx = (struct schan_context *)&conn;

    memset(&cred, 0, sizeof(cred));
    cred.credential_use = SECPKG_CRED_OUTBOUND;
    cred.enabled_protocols = 0;

    if (!schan_imp_create_session(&session, &cred)) { rc = -5; goto done; }
    schan_imp_set_session_transport(session, &transport);
    schan_imp_set_session_target(session, "schan-harness.invalid");

    do {
        st = schan_imp_handshake(session);
        r->handshake_iterations++;
        if (st == SEC_I_CONTINUE_NEEDED && wait_readable(sv[0]) <= 0) break;
    } while (st == SEC_I_CONTINUE_NEEDED && r->handshake_iterations < MAX_HANDSHAKE_STEPS);
    r->handshake_status = (int)st;
    if (st != SEC_E_OK) { rc = -6; goto done; }

    r->block_size = schan_imp_get_session_cipher_block_size(session);
    r->max_message_size = schan_imp_get_max_message_size(session);

    memset(&info, 0, sizeof(info));
    r->conn_info_status = (int)schan_imp_get_connection_info(session, &info);
    r->conn_protocol = info.dwProtocol;
    r->conn_cipher = info.aiCipher;
    r->conn_cipher_strength = info.dwCipherStrength;
    r->conn_hash = info.aiHash;
    r->conn_hash_strength = info.dwHashStrength;
    r->conn_exch = info.aiExch;
    r->conn_exch_strength = info.dwExchStrength;

    r->peer_cert_status = (int)schan_imp_get_session_peer_certificate(session, NULL, &cert);
    r->peer_cert_len = g_peer_der_len;
    r->peer_cert_matches = (g_peer_der_len == schan_harness_crt_der_len) &&
                           !memcmp(g_peer_der, schan_harness_crt_der, g_peer_der_len);

    len = strlen(msg);
    if (schan_imp_send(session, msg, &len) != SEC_E_OK) { rc = -7; goto done; }

    memset(got, 0, sizeof(got));
    {
        SIZE_T total = 0;
        int spins = 0;
        while (total < strlen(msg) && spins++ < MAX_HANDSHAKE_STEPS)
        {
            SIZE_T want = sizeof(got) - 1 - total;
            SECURITY_STATUS rs = schan_imp_recv(session, got + total, &want);
            if (rs == SEC_E_OK) total += want;
            else if (rs != SEC_I_CONTINUE_NEEDED) break;
            else if (wait_readable(sv[0]) <= 0) break;
        }
        r->echo_ok = (total == strlen(msg));
    }
    memcpy(r->echo_got, got, sizeof(r->echo_got) - 1);
    rc = 0;

done:
    if (session) schan_imp_dispose_session(session);
    shutdown(sv[0], SHUT_RDWR);
    close(sv[0]);
    pthread_join(th, NULL);
    close(sv[1]);
    r->server_err = sa.err;
    schan_imp_deinit();
    return rc;
}
