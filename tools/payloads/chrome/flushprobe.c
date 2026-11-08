/* flushprobe: does FlushFileBuffers cost grow with the file's size?
 * Chrome runs with --enable-logging --log-file, which flushes after every line. A thread of it
 * was found inside NtFlushBuffersFile each time the sweep's GetThreadContext on it took over 20 s
 * (a flush holds the file system's locks at APC_LEVEL, so the context APC waits it out). If each
 * flush rewrites the whole file, a growing log makes every flush slower and the thread is nearly
 * always inside one. This appends 4000 lines of 200 bytes, flushing after each, and prints the
 * milliseconds per block of 500: flat on Windows, growing if flush cost tracks file size.
 * One line per fact, to diff with flushprobe.win11.txt (timings differ; the shape is the fact). */
#include <windows.h>
#include <stdio.h>

#define LINES 4000
#define BLOCK 500

int main(int argc, char **argv)
{
    const char *path = argc > 1 ? argv[1] : "C:\\TEMP\\flushprobe.log";
    HANDLE h = CreateFileA(path, GENERIC_WRITE, FILE_SHARE_READ, NULL, CREATE_ALWAYS,
                           FILE_ATTRIBUTE_NORMAL, NULL);
    if (h == INVALID_HANDLE_VALUE) {
        printf("[FLUSH] cannot create %s: %lu\n", path, GetLastError());
        return 1;
    }
    char line[200];
    memset(line, 'x', sizeof line);
    line[sizeof line - 1] = '\n';
    DWORD first = 0, last = 0, fails = 0;
    LARGE_INTEGER f, t0, t1;
    QueryPerformanceFrequency(&f);
    QueryPerformanceCounter(&t0);
    for (int i = 1; i <= LINES; i++) {
        DWORD n;
        if (!WriteFile(h, line, sizeof line, &n, NULL) || !FlushFileBuffers(h))
            fails++;
        if (i % BLOCK == 0) {
            QueryPerformanceCounter(&t1);
            DWORD ms = (DWORD)((t1.QuadPart - t0.QuadPart) * 1000 / f.QuadPart);
            printf("[FLUSH] lines %d-%d (file %d KB): %lu ms\n", i - BLOCK + 1, i, i * (int)sizeof line / 1024, ms);
            fflush(stdout);
            if (!first)
                first = ms ? ms : 1;
            last = ms ? ms : 1;
            t0 = t1;
        }
    }
    CloseHandle(h);
    DeleteFileA(path);
    printf("[FLUSH] last block / first block: %s (x%lu.%lu), failures %lu\n",
           last > first * 3 ? "GREW" : "flat", (unsigned long)(last / first),
           (unsigned long)(last * 10 / first % 10), (unsigned long)fails);
    printf("[FLUSH] done\n");
    return 0;
}
