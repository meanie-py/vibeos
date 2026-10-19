struct _stat { int x; };
#undef erased
int msvcrt_stat64_to_stat(int x) { return x; }
int _stat(void) { return 0; }
void *pointer(void) { return 0; }
int WINAPIV variadic(int x, ...) { return x; }
int erased() { return 0; }
