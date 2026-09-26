/* Host grading only for the compiler-generated JScript decode.c candidate.
 * Prototypes match sdk/include/wine/debug.h and the extracted wcsncmp ABI.
 * Production must link the real Wine diagnostics and CRT functions.
 */
#include <stddef.h>
#include <stdint.h>

typedef uint16_t WCHAR;

enum __wine_debug_class {
    __WINE_DBCL_FIXME = 0,
    __WINE_DBCL_ERR = 1,
    __WINE_DBCL_WARN = 2,
    __WINE_DBCL_TRACE = 3,
    __WINE_DBCL_INIT = 7
};

struct __wine_debug_channel {
    unsigned char flags;
    char name[15];
};

static unsigned long diagnostic_count;
static unsigned long diagnostic_count_by_class[8];

/* Linux libc's wchar_t is 32 bits, while guest WCHAR and the anchor's
 * wchar_t are 16 bits. Use an exact-width host comparator. */
int wcsncmp(const WCHAR *left, const WCHAR *right, size_t count)
{
    for (size_t index = 0; index < count; ++index) {
        if (left[index] != right[index])
            return left[index] < right[index] ? -1 : 1;
        if (left[index] == 0)
            return 0;
    }
    return 0;
}

/* decode.c passes this result only to ros_dbg_log. The diagnostic text has
 * no influence on the return code or in-place decode. */
const char *wine_dbgstr_wn(const WCHAR *text, int count)
{
    (void)text;
    (void)count;
    return "<host-wide-diagnostic>";
}

int ros_dbg_log(enum __wine_debug_class cls, struct __wine_debug_channel *channel,
                const char *file, const char *function, int line,
                const char *format, ...)
{
    (void)channel;
    (void)file;
    (void)function;
    (void)line;
    (void)format;
    ++diagnostic_count;
    if ((unsigned int)cls < 8)
        ++diagnostic_count_by_class[(unsigned int)cls];
    return 0;
}

unsigned long decode_host_diagnostic_count(void)
{
    return diagnostic_count;
}

unsigned long decode_host_diagnostic_count_for_class(unsigned int cls)
{
    return cls < 8 ? diagnostic_count_by_class[cls] : 0;
}
