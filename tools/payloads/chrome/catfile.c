/* catfile: print files to stdout, one "[CATFILE] <path>" header each, so the sweep runner carries
 * a log a program wrote (Chrome's --log-file; its children's stderr never reaches the pipe) to
 * the serial after that program has exited or been killed. */
#include <stdio.h>

int main(int argc, char **argv)
{
    char line[4096];
    int i, rc = 0;
    for (i = 1; i < argc; i++) {
        FILE *f = fopen(argv[i], "rb");
        printf("[CATFILE] %s%s\n", argv[i], f ? "" : " (cannot open)");
        if (!f) { rc = 1; continue; }
        while (fgets(line, sizeof(line), f))
            fputs(line, stdout);
        fclose(f);
        printf("[CATFILE] end %s\n", argv[i]);
    }
    return rc;
}
