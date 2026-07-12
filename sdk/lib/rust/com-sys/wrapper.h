/* com-sys bindgen wrapper — COM interface vtbls + automation types.
 *
 * Order matters: objbase.h must come before the interface headers (it defines
 * `interface`/BEGIN_INTERFACE; including unknwn.h bare fails to parse).
 * Regenerate with ./generate_bindings.sh — see that script for the flag set.
 */
#define WIN32_NO_STATUS
#define COBJMACROS
#include <windef.h>
#include <winbase.h>
#include <objbase.h>
#include <oleidl.h>
#include <servprov.h>
#include <oaidl.h>
#include <ocidl.h>
#include <propidl.h>
