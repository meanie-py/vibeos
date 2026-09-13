/*
 * sqlite32.exe — an i386 WoW64 stress guest built around real SQLite.
 *
 * The existing guests each probe ONE mechanism deliberately (a syscall, a
 * marshaller, SEH). This one is the opposite: a large third-party C program
 * that was never written with us in mind, doing what it normally does. It
 * leans on the parts of the 32-bit substrate the probes barely touch --
 * allocator churn under a real pager, Win32 file I/O with locking and
 * truncation, and long stretches of pure compute between syscalls.
 *
 * Verdict contract (same as hello32/file32/thread32): 0x600D on PASS, a
 * distinct 0xBADn per failing rung so a red run names the rung on serial
 * rather than just "it died".
 *
 * Deliberately MODEST in size: this runs as x86 under FEX under TCG, so the
 * row counts here are chosen to stress the substrate without turning a
 * ladder boot into a coffee break.
 */
#include <windows.h>
#include "sqlite3.h"

#define SQL32_PASS          0x600D
#define SQL32_BAD_OPEN      0xBAD1
#define SQL32_BAD_CREATE    0xBAD2
#define SQL32_BAD_INSERT    0xBAD3
#define SQL32_BAD_COUNT     0xBAD4
#define SQL32_BAD_INTEGRITY 0xBAD5
#define SQL32_BAD_BLOB      0xBAD6
#define SQL32_BAD_CLOSE     0xBAD7

#define ROWS 400

static void Say(const char *s)
{
    /* OutputDebugStringA lands on the serial log through the same path the
     * other guests use, so a failure is readable without a debugger. */
    OutputDebugStringA(s);
}

static int Exec(sqlite3 *db, const char *sql)
{
    char *err = 0;
    int rc = sqlite3_exec(db, sql, 0, 0, &err);
    if (rc != SQLITE_OK) {
        Say("[sqlite32] SQL error: ");
        Say(err ? err : "(null)");
        Say("\n");
        if (err) sqlite3_free(err);
    }
    return rc;
}

int main(void)
{
    sqlite3 *db = 0;
    sqlite3_stmt *st = 0;
    char path[MAX_PATH];
    int i, rc, n = 0, on_disk = 0;

    Say("[sqlite32] start\n");

    /* On disk, not :memory: -- the point is to exercise the Win32 file path
     * (CreateFile/ReadFile/WriteFile/SetEndOfFile/LockFile) through wow64.
     *
     * Derived from GetWindowsDirectory, like file32.exe, and NOT from
     * GetTempPath: the amd64 livecd's system volume is X: and the arm64
     * ramdisk's is C:, and TEMP on the livecd answers CreateFile with
     * STATUS_ACCESS_DENIED while syswow64 is writable on both. Using the temp
     * dir cost this guest an 0xBAD1 on its first amd64 run -- the same trap
     * file32.exe documents. */
    {
        int i = 0;
        GetWindowsDirectoryA(path, MAX_PATH - 32);
        while (path[i]) i++;
        if (i && path[i - 1] != '\\') path[i++] = '\\';
        path[i] = 0;
        lstrcatA(path, "syswow64\\sqlite32.db");
    }
    DeleteFileA(path);

    /* The amd64 livecd volume is READ-ONLY -- file32.exe hits the same wall and
     * skips its write rung there ("write rung SKIPPED (read-only volume)").
     * Rather than skip outright, fall back to an in-memory database so the
     * allocator, B-tree, sorter and SQL engine still get exercised on that
     * media; only the file-I/O and locking axis is lost, and serial says so.
     * Probe with CreateFile rather than guessing from the drive letter. */
    {
        HANDLE probe = CreateFileA(path, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS,
                                   FILE_ATTRIBUTE_NORMAL, NULL);
        if (probe == INVALID_HANDLE_VALUE) {
            Say("[sqlite32] volume is read-only -- using an IN-MEMORY db "
                "(file I/O + locking NOT covered on this media)\n");
            lstrcpyA(path, ":memory:");
        } else {
            CloseHandle(probe);
            DeleteFileA(path);
            on_disk = 1;
        }
    }

    if (sqlite3_open(path, &db) != SQLITE_OK) { Say("[sqlite32] FAIL open\n"); return SQL32_BAD_OPEN; }
    Say(on_disk ? "[sqlite32] opened db on disk\n" : "[sqlite32] opened db in memory\n");

    if (on_disk && Exec(db, "PRAGMA journal_mode=DELETE;") != SQLITE_OK) return SQL32_BAD_CREATE;
    if (Exec(db, "CREATE TABLE t(id INTEGER PRIMARY KEY, name TEXT, val REAL, data BLOB);") != SQLITE_OK)
        return SQL32_BAD_CREATE;

    /* Bulk insert in one transaction: allocator churn + page cache + journal. */
    if (Exec(db, "BEGIN;") != SQLITE_OK) return SQL32_BAD_INSERT;
    rc = sqlite3_prepare_v2(db, "INSERT INTO t(name,val,data) VALUES(?,?,?);", -1, &st, 0);
    if (rc != SQLITE_OK) return SQL32_BAD_INSERT;
    for (i = 0; i < ROWS; i++) {
        char nm[32];
        unsigned char blob[64];
        int j;
        wsprintfA(nm, "row-%d", i);
        for (j = 0; j < (int)sizeof(blob); j++) blob[j] = (unsigned char)(i + j);
        sqlite3_bind_text(st, 1, nm, -1, SQLITE_TRANSIENT);
        sqlite3_bind_double(st, 2, i * 1.5);
        sqlite3_bind_blob(st, 3, blob, sizeof(blob), SQLITE_TRANSIENT);
        if (sqlite3_step(st) != SQLITE_DONE) { sqlite3_finalize(st); return SQL32_BAD_INSERT; }
        sqlite3_reset(st);
    }
    sqlite3_finalize(st);
    if (Exec(db, "COMMIT;") != SQLITE_OK) return SQL32_BAD_INSERT;
    Say("[sqlite32] rung1 PASS: bulk insert committed\n");

    /* Index build + aggregate query: sorting, B-tree churn, float math. */
    if (Exec(db, "CREATE INDEX idx_val ON t(val);") != SQLITE_OK) return SQL32_BAD_CREATE;
    rc = sqlite3_prepare_v2(db, "SELECT COUNT(*) FROM t WHERE val > 10.0;", -1, &st, 0);
    if (rc != SQLITE_OK) return SQL32_BAD_COUNT;
    if (sqlite3_step(st) == SQLITE_ROW) n = sqlite3_column_int(st, 0);
    sqlite3_finalize(st);
    if (n != ROWS - 7) {   /* val = i*1.5 > 10.0  <=>  i >= 7 */
        Say("[sqlite32] FAIL: aggregate count wrong\n");
        return SQL32_BAD_COUNT;
    }
    Say("[sqlite32] rung2 PASS: index + aggregate query\n");

    /* Blob round-trip: proves byte-exact data survived the file layer. */
    rc = sqlite3_prepare_v2(db, "SELECT data FROM t WHERE id=100;", -1, &st, 0);
    if (rc != SQLITE_OK) return SQL32_BAD_BLOB;
    if (sqlite3_step(st) == SQLITE_ROW) {
        const unsigned char *p = (const unsigned char *)sqlite3_column_blob(st, 0);
        int len = sqlite3_column_bytes(st, 0);
        int k;
        if (len != 64) { sqlite3_finalize(st); return SQL32_BAD_BLOB; }
        for (k = 0; k < len; k++) {
            if (p[k] != (unsigned char)(99 + k)) { sqlite3_finalize(st); return SQL32_BAD_BLOB; }
        }
    } else { sqlite3_finalize(st); return SQL32_BAD_BLOB; }
    sqlite3_finalize(st);
    Say("[sqlite32] rung3 PASS: blob round-tripped byte-exact\n");

    /* Delete half the rows then VACUUM: file truncation + rewrite. */
    if (Exec(db, "DELETE FROM t WHERE id % 2 = 0;") != SQLITE_OK) return SQL32_BAD_INSERT;
    if (Exec(db, "VACUUM;") != SQLITE_OK) return SQL32_BAD_INSERT;
    Say("[sqlite32] rung4 PASS: delete + VACUUM rewrote the file\n");

    /* SQLite's own verdict on the file it just wrote. */
    rc = sqlite3_prepare_v2(db, "PRAGMA integrity_check;", -1, &st, 0);
    if (rc != SQLITE_OK) return SQL32_BAD_INTEGRITY;
    if (sqlite3_step(st) == SQLITE_ROW) {
        const char *res = (const char *)sqlite3_column_text(st, 0);
        if (!res || lstrcmpA(res, "ok") != 0) {
            Say("[sqlite32] FAIL integrity_check: ");
            Say(res ? res : "(null)");
            Say("\n");
            sqlite3_finalize(st);
            return SQL32_BAD_INTEGRITY;
        }
    } else { sqlite3_finalize(st); return SQL32_BAD_INTEGRITY; }
    sqlite3_finalize(st);
    Say("[sqlite32] rung5 PASS: integrity_check ok\n");

    if (sqlite3_close(db) != SQLITE_OK) return SQL32_BAD_CLOSE;
    if (on_disk) DeleteFileA(path);

    Say("[sqlite32] ALL PASS\n");
    return SQL32_PASS;
}
