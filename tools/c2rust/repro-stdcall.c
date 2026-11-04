/* Stock c2rust 0.22.1 panics transpiling this ("ctype parameter must be a function instead
 * of Attributed"); the patched build in install.sh transpiles it. rustconv's c2rust_anchor
 * runs it as the probe that refuses a stock binary. */
int __attribute__((stdcall)) f(int a) { return a; }
typedef int (__attribute__((stdcall)) *PFN)(int);
PFN get(void) { return f; }
