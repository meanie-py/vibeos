/* SAHF with AH = 0x29 (CF set, ZF clear: what fnstsw gives for 0 < 1.0 at TOP=5) after
 * different flag-setting integer ops; jbe must be TAKEN (1) every time. Pure integer code,
 * no x87, so a wrong answer here is the CPU (emulator), not anything in the FPU. */
#define SAHFV(name, pre) \
static unsigned name(void) { unsigned r, t = 0x80000000u, u = 0x12345678u; \
    __asm__ volatile("movl %2, %%edx\n\t" pre "\n\tmovb $0x29, %%ah\n\tsahf\n\tmovl $1, %0\n\tjbe 1f\n\tmovl $0, %0\n1:\n\t" \
        : "=&r"(r), "+r"(u) : "r"(t) : "eax", "edx", "cc"); return r; }
SAHFV(s_none, "")
SAHFV(s_or, "orl %1, %%edx")
SAHFV(s_or0, "orl $0, %%edx")
SAHFV(s_and, "andl %1, %%edx")
SAHFV(s_xor, "xorl %1, %%edx")
SAHFV(s_add, "addl %1, %%edx")
SAHFV(s_sub, "subl %1, %%edx")
SAHFV(s_sbb, "subl %1, %%edx\n\tsbbl %1, %%edx")
SAHFV(s_inc, "incl %%edx")
SAHFV(s_cmp, "cmpl %1, %%edx")
SAHFV(s_test, "testl %%edx, %%edx")
SAHFV(s_shl, "shll $1, %%edx")
SAHFV(s_neg, "negl %%edx")
static unsigned (*const sahfv[])(void) = { s_none, s_or, s_or0, s_and, s_xor, s_add, s_sub, s_sbb, s_inc, s_cmp, s_test, s_shl, s_neg };
static const char *const sahfn[] = { "none", "or", "or0", "and", "xor", "add", "sub", "sbb", "inc", "cmp", "test", "shl", "neg" };
