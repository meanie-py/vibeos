/* grepfile: print the lines of a file that contain any of the given strings.
 * Usage: grepfile <file> <max lines> <string> [string...]
 *
 * catfile prints a whole log, and some logs are too big for the serial line: Chrome's net log
 * (--log-net-log) is megabytes of one-event-per-line JSON. This prints just the matching lines,
 * each cut to 400 characters, at most <max lines> of them, then how many matched. */
#include <windows.h>
#include <stdio.h>
#include <string.h>

int main(int argc, char **argv)
{
    if (argc < 4) {
        printf("[GREPFILE] usage: grepfile <file> <max lines> <string>...\n");
        return 2;
    }
    FILE *f = fopen(argv[1], "rb");
    if (!f) {
        printf("[GREPFILE] cannot open %s (error %lu)\n", argv[1], GetLastError());
        return 1;
    }
    long max = atol(argv[2]), shown = 0, matched = 0, lines = 0;
    static char line[1 << 20];
    printf("[GREPFILE] %s\n", argv[1]);
    while (fgets(line, sizeof line, f)) {
        size_t n = strlen(line);
        /* A line longer than the buffer: drop its tail (it is tested by its head only). */
        if (n == sizeof line - 1 && line[n - 1] != '\n') {
            int ch;
            while ((ch = fgetc(f)) != EOF && ch != '\n') {
            }
        }
        lines++;
        while (n && (line[n - 1] == '\n' || line[n - 1] == '\r'))
            line[--n] = 0;
        for (int i = 3; i < argc; i++) {
            if (strstr(line, argv[i])) {
                matched++;
                if (shown < max) {
                    printf("%.400s\n", line);
                    shown++;
                }
                break;
            }
        }
    }
    fclose(f);
    printf("[GREPFILE] end: %ld of %ld lines matched, %ld shown\n", matched, lines, shown);
    fflush(stdout);
    return 0;
}
