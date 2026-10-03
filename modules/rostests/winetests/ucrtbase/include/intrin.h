/* Shadows dll/win32/msvcrt/include/intrin.h, Wine's, which is next on this
 * test's include path and does not coexist with clang's x86 intrinsic headers
 * (fma4intrin.h: unknown type '__m256'). Everything else in Wine's CRT header
 * set is what these tests are written against; the intrinsics are ReactOS's. */
#include "../../../../../sdk/include/vcruntime/intrin.h"
