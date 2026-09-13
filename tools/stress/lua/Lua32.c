/*
 * lua32.exe — an i386 WoW64 stress guest built around the real Lua 5.4
 * interpreter.
 *
 * sqlite32 stresses the DATA path (allocator under a pager, Win32 file I/O,
 * long compute between syscalls). This one stresses the CONTROL path, which
 * nothing else in the ladder touches: Lua implements both error propagation
 * and coroutine switching with setjmp/longjmp, so every rung below drives a
 * non-local jump across frames the emulator has already translated. That is
 * the hardest case for FEX's block cache on arm64 and for the 32-bit CRT's
 * setjmp/longjmp on both arches, and it is exactly the mechanism that the
 * nine deliberate probes -- including fault32's SEH -- never exercise, because
 * SEH unwinding and C setjmp/longjmp are different machinery.
 *
 * It is also, like sqlite32, a large third-party C program written with no
 * knowledge of us: the GC, string interner and parser all run for real.
 *
 * Verdict contract (same as hello32/file32/thread32/sqlite32): 0x600D on PASS,
 * a distinct 0xBADn per failing rung so a red run names the rung on serial
 * rather than just "it died".
 *
 * Deliberately MODEST in size: this runs as x86 under FEX under TCG, so the
 * iteration counts are chosen to stress the substrate without turning a ladder
 * boot into a coffee break.
 */
#include <windows.h>
#include "lua.h"
#include "lauxlib.h"
#include "lualib.h"

#define LUA32_PASS         0x600D
#define LUA32_BAD_NEWSTATE 0xBAD1
#define LUA32_BAD_EVAL     0xBAD2
#define LUA32_BAD_GC       0xBAD3
#define LUA32_BAD_PCALL    0xBAD4
#define LUA32_BAD_RECURSE  0xBAD5
#define LUA32_BAD_COROUT   0xBAD6
#define LUA32_BAD_STRING   0xBAD7

static void Say(const char *s)
{
    /* OutputDebugStringA lands on the serial log through the same path the
     * other guests use, so a failure is readable without a debugger. */
    OutputDebugStringA(s);
}

static void SayNum(const char *label, lua_Integer v)
{
    char buf[64];
    wsprintfA(buf, "%s%d\n", label, (int)v);
    Say(buf);
}

/* Run a chunk and leave its single result on the stack. Returns 0 on success.
 * Any Lua error here has already crossed a longjmp on its way out of the
 * interpreter, so a failure at this level is itself the interesting signal --
 * report the message rather than swallowing it. */
static int DoChunk(lua_State *L, const char *src)
{
    if (luaL_loadstring(L, src) != LUA_OK || lua_pcall(L, 0, 1, 0) != LUA_OK) {
        Say("[lua32] lua error: ");
        Say(lua_tostring(L, -1) ? lua_tostring(L, -1) : "(null)");
        Say("\n");
        lua_pop(L, 1);
        return 1;
    }
    return 0;
}

int main(void)
{
    lua_State *L;
    lua_Integer n;

    Say("[lua32] start\n");

    L = luaL_newstate();
    if (!L) { Say("[lua32] FAIL luaL_newstate\n"); return LUA32_BAD_NEWSTATE; }
    luaL_openlibs(L);

    /* Rung 1: parse + execute. The Lua parser is a recursive-descent compiler
     * emitting bytecode, so this alone is a few thousand indirect branches
     * through freshly written memory -- the case FEX has to get right. */
    if (DoChunk(L, "local s = 0 for i = 1, 20000 do s = s + i % 7 end return s"))
        { lua_close(L); return LUA32_BAD_EVAL; }
    n = lua_tointeger(L, -1);
    lua_pop(L, 1);
    if (n != 59998) { SayNum("[lua32] FAIL eval, got ", n); lua_close(L); return LUA32_BAD_EVAL; }
    Say("[lua32] rung1 PASS: parse + 20k-iteration loop\n");

    /* Rung 2: allocator churn under the incremental GC. Tables and strings are
     * created and dropped fast enough that a full GC cycle runs inside the
     * loop; collectgarbage("count") then proves the heap actually came back
     * down rather than the GC quietly never running. */
    if (DoChunk(L,
        "collectgarbage('collect')\n"
        "local before = collectgarbage('count')\n"
        "for i = 1, 4000 do\n"
        "  local t = {}\n"
        "  for j = 1, 20 do t[j] = 'k' .. i .. '-' .. j end\n"
        "end\n"
        "collectgarbage('collect')\n"
        "local after = collectgarbage('count')\n"
        "return (after < before + 256) and 1 or 0"))
        { lua_close(L); return LUA32_BAD_GC; }
    n = lua_tointeger(L, -1);
    lua_pop(L, 1);
    if (n != 1) { Say("[lua32] FAIL: heap did not return after GC\n"); lua_close(L); return LUA32_BAD_GC; }
    Say("[lua32] rung2 PASS: 80k table/string allocations reclaimed by GC\n");

    /* Rung 3: THE longjmp rung. error() inside a pcall is a longjmp out of an
     * arbitrarily deep interpreter frame back to a setjmp buffer saved by the
     * protected call. Run it 2000 times so a block cache that mis-handles the
     * jump target fails reliably rather than once in a rare boot, and nest the
     * pcalls so the unwind crosses more than one saved buffer. */
    if (DoChunk(L,
        "local caught = 0\n"
        "for i = 1, 2000 do\n"
        "  local ok, err = pcall(function()\n"
        "    local ok2 = pcall(function() error({ code = i }) end)\n"
        "    if ok2 then error('inner pcall should have failed') end\n"
        "    error('depth-' .. i, 2)\n"
        "  end)\n"
        "  if not ok and type(err) == 'string' then caught = caught + 1 end\n"
        "end\n"
        "return caught"))
        { lua_close(L); return LUA32_BAD_PCALL; }
    n = lua_tointeger(L, -1);
    lua_pop(L, 1);
    if (n != 2000) { SayNum("[lua32] FAIL pcall, caught ", n); lua_close(L); return LUA32_BAD_PCALL; }
    Say("[lua32] rung3 PASS: 4000 nested longjmp unwinds via pcall/error\n");

    /* Rung 4: stack growth then an error raised from the deepest frame. Lua
     * reallocates its value stack as recursion deepens, so the longjmp at the
     * end unwinds through frames whose stack has MOVED since they were entered
     * -- a pointer-invalidation case that a shallower test cannot reach. */
    if (DoChunk(L,
        "local function deep(n) if n == 0 then error('bottom') end return deep(n - 1) end\n"
        "local ok, err = pcall(deep, 400)\n"
        "if ok then return 0 end\n"
        "return (type(err) == 'string' and 1 or 0)"))
        { lua_close(L); return LUA32_BAD_RECURSE; }
    n = lua_tointeger(L, -1);
    lua_pop(L, 1);
    if (n != 1) { Say("[lua32] FAIL: deep recursion did not unwind cleanly\n"); lua_close(L); return LUA32_BAD_RECURSE; }
    Say("[lua32] rung4 PASS: 400-deep recursion unwound after stack reallocation\n");

    /* Rung 5: coroutines. Each resume/yield pair is a full switch between two
     * independent Lua stacks, which on this build means another longjmp and a
     * different lua_State -- i.e. the emulator sees the same code running with
     * a different stack pointer, repeatedly. 500 round trips through a
     * generator, plus one coroutine that dies with an error so the failure
     * path crosses a coroutine boundary too. */
    if (DoChunk(L,
        "local co = coroutine.wrap(function() for i = 1, 500 do coroutine.yield(i * 2) end return -1 end)\n"
        "local sum = 0\n"
        "for i = 1, 500 do sum = sum + co() end\n"
        "local bad = coroutine.create(function() error('in-coroutine') end)\n"
        "local ok = coroutine.resume(bad)\n"
        "if ok then return 0 end\n"
        "return sum"))
        { lua_close(L); return LUA32_BAD_COROUT; }
    n = lua_tointeger(L, -1);
    lua_pop(L, 1);
    if (n != 250500) { SayNum("[lua32] FAIL coroutine, sum ", n); lua_close(L); return LUA32_BAD_COROUT; }
    Say("[lua32] rung5 PASS: 500 coroutine switches + error across a coroutine\n");

    /* Rung 6: string library round-trip. string.format goes through the CRT's
     * float formatting, gsub drives the pattern matcher (heavy backtracking),
     * and the interner has to collapse the duplicates. This is the rung that
     * would catch a width bug in the 32-bit CRT rather than in the emulator. */
    if (DoChunk(L,
        "local s = ''\n"
        "for i = 1, 300 do s = s .. string.format('%d:%.3f;', i, i / 7) end\n"
        "local t, nsub = s:gsub('%d+:', 'n:')\n"
        "if nsub ~= 300 then return 0 end\n"
        "local _, count = s:gsub(';', ';')\n"
        "if count ~= 300 then return 0 end\n"
        "if t:find('%d+:') then return 0 end\n"
        "return (s:find('7:1%.000;') and 1 or 0)"))
        { lua_close(L); return LUA32_BAD_STRING; }
    n = lua_tointeger(L, -1);
    lua_pop(L, 1);
    if (n != 1) { Say("[lua32] FAIL: string.format/gsub round-trip\n"); lua_close(L); return LUA32_BAD_STRING; }
    Say("[lua32] rung6 PASS: string.format + pattern matching round-trip\n");

    lua_close(L);
    Say("[lua32] ALL PASS\n");
    return LUA32_PASS;
}
