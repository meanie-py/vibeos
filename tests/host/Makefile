# ReactOS Host-Side Test Runner
#
# Compiles GREEN-category kernel algorithm tests natively on macOS/Linux.
# Uses the actual RTL source code from the ReactOS tree with a thin
# host shim that provides NT type definitions.
#
# Usage:
#   make           # build
#   make test      # build and run
#   make clean     # clean build artifacts

CC ?= clang
CFLAGS = -std=c11 -Wall -Wextra -Wno-unused-parameter -Wno-sign-compare \
         -Wno-unused-variable -Wno-unused-but-set-variable \
         -Wno-pointer-sign -Wno-macro-redefined \
         -fshort-wchar -g -O1 \
         -D'__declspec(x)=' -D__stdcall=

# Our host shim headers take priority over ReactOS headers
HOST_INCLUDE = -I$(CURDIR)/include

# ReactOS source tree (relative to this Makefile)
REACTOS = ../..

# Source paths
RTL_SRC = $(REACTOS)/sdk/lib/rtl
FSRTL_SRC = $(REACTOS)/ntoskrnl/fsrtl
KE_SRC = $(REACTOS)/ntoskrnl/ke
TEST_SRC = $(REACTOS)/modules/rostests/kmtests

# Compile flags per source category
RTL_CFLAGS = $(CFLAGS) $(HOST_INCLUDE) -DNDEBUG
KERNEL_CFLAGS = $(CFLAGS) $(HOST_INCLUDE) -DNDEBUG
TEST_CFLAGS = $(CFLAGS) $(HOST_INCLUDE) -DNDEBUG -Wno-format

# Output
BUILD_DIR = build
TARGET = $(BUILD_DIR)/run_tests

# Object files
OBJS = \
	$(BUILD_DIR)/splaytree.o \
	$(BUILD_DIR)/generictable.o \
	$(BUILD_DIR)/avlsupp_avltable.o \
	$(BUILD_DIR)/rangelist.o \
	$(BUILD_DIR)/largemcb.o \
	$(BUILD_DIR)/tunnel.o \
	$(BUILD_DIR)/fsrtl_dots.o \
	$(BUILD_DIR)/fsrtl_dbcsname.o \
	$(BUILD_DIR)/fsrtl_name.o \
	$(BUILD_DIR)/fsrtl_data.o \
	$(BUILD_DIR)/nls_stub.o \
	$(BUILD_DIR)/seh_host.o \
	$(BUILD_DIR)/test_RtlSplayTree.o \
	$(BUILD_DIR)/test_RtlAvlTree.o \
	$(BUILD_DIR)/test_FsRtlDissect.o \
	$(BUILD_DIR)/test_FsRtlLegal.o \
	$(BUILD_DIR)/test_FsRtlExpression.o \
	$(BUILD_DIR)/test_RtlMemory.o \
	$(BUILD_DIR)/test_ExDoubleList.o \
	$(BUILD_DIR)/test_ExSingleList.o \
	$(BUILD_DIR)/test_RtlUnicodeString.o \
	$(BUILD_DIR)/test_RtlRangeList.o \
	$(BUILD_DIR)/test_FsRtlMcb.o \
	$(BUILD_DIR)/test_FsRtlTunnel.o \
	$(BUILD_DIR)/test_FsRtlRemoveDotsFromPath.o \
	$(BUILD_DIR)/test_RtlException.o \
	$(BUILD_DIR)/test_ExUuid.o \
	$(BUILD_DIR)/test_ExInterlocked.o \
	$(BUILD_DIR)/devqueue.o \
	$(BUILD_DIR)/test_KeDeviceQueue.o \
	$(BUILD_DIR)/test_KeIrql.o \
	$(BUILD_DIR)/test_GuardedMemory.o \
	$(BUILD_DIR)/test_runner.o

.PHONY: all test clean

all: $(TARGET)

test: $(TARGET)
	@$(TARGET)

$(TARGET): $(OBJS) | $(BUILD_DIR)
	$(CC) $(CFLAGS) -o $@ $(OBJS)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# --- RTL implementation objects ---

$(BUILD_DIR)/splaytree.o: $(RTL_SRC)/splaytree.c | $(BUILD_DIR)
	$(CC) $(RTL_CFLAGS) -c -o $@ $<

$(BUILD_DIR)/generictable.o: $(RTL_SRC)/generictable.c | $(BUILD_DIR)
	$(CC) $(RTL_CFLAGS) -c -o $@ $<

$(BUILD_DIR)/avlsupp_avltable.o: $(RTL_SRC)/avltable.c $(RTL_SRC)/avlsupp.c $(RTL_SRC)/rtlavl.h | $(BUILD_DIR)
	$(CC) $(RTL_CFLAGS) -I$(RTL_SRC) -c -o $@ $<

$(BUILD_DIR)/rangelist.o: $(RTL_SRC)/rangelist.c | $(BUILD_DIR)
	$(CC) $(RTL_CFLAGS) -c -o $@ $<

$(BUILD_DIR)/largemcb.o: $(FSRTL_SRC)/largemcb.c | $(BUILD_DIR)
	$(CC) $(KERNEL_CFLAGS) -c -o $@ $<

$(BUILD_DIR)/tunnel.o: $(FSRTL_SRC)/tunnel.c | $(BUILD_DIR)
	$(CC) $(KERNEL_CFLAGS) -c -o $@ $<

$(BUILD_DIR)/fsrtl_dots.o: fsrtl_dots.c | $(BUILD_DIR)
	$(CC) $(CFLAGS) -I$(CURDIR)/include -c -o $@ $<

# --- Kernel FsRtl implementation objects ---

$(BUILD_DIR)/fsrtl_dbcsname.o: $(FSRTL_SRC)/dbcsname.c | $(BUILD_DIR)
	$(CC) $(KERNEL_CFLAGS) -c -o $@ $<

$(BUILD_DIR)/fsrtl_name.o: $(FSRTL_SRC)/name.c | $(BUILD_DIR)
	$(CC) $(KERNEL_CFLAGS) -c -o $@ $<

# --- Host support ---

$(BUILD_DIR)/fsrtl_data.o: fsrtl_data.c | $(BUILD_DIR)
	$(CC) $(CFLAGS) -I$(CURDIR)/include -c -o $@ $<

$(BUILD_DIR)/nls_stub.o: nls_stub.c | $(BUILD_DIR)
	$(CC) $(CFLAGS) -I$(CURDIR)/include -c -o $@ $<

$(BUILD_DIR)/seh_host.o: seh_host.c | $(BUILD_DIR)
	$(CC) $(CFLAGS) -I$(CURDIR)/include -c -o $@ $<

# --- Test objects ---

$(BUILD_DIR)/test_RtlSplayTree.o: $(TEST_SRC)/rtl/RtlSplayTree.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -c -o $@ $<

$(BUILD_DIR)/test_RtlAvlTree.o: $(TEST_SRC)/rtl/RtlAvlTree.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -c -o $@ $<

$(BUILD_DIR)/test_FsRtlDissect.o: $(TEST_SRC)/ntos_fsrtl/FsRtlDissect.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -c -o $@ $<

$(BUILD_DIR)/test_FsRtlLegal.o: $(TEST_SRC)/ntos_fsrtl/FsRtlLegal.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -c -o $@ $<

$(BUILD_DIR)/test_FsRtlExpression.o: $(TEST_SRC)/ntos_fsrtl/FsRtlExpression.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -c -o $@ $<

$(BUILD_DIR)/test_RtlMemory.o: $(TEST_SRC)/rtl/RtlMemory.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -c -o $@ $<

# -O0: setjmp/longjmp SEH requires unoptimized code (locals across setjmp must be volatile)
$(BUILD_DIR)/test_RtlUnicodeString.o: $(TEST_SRC)/rtl/RtlUnicodeString.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -O0 -c -o $@ $<

$(BUILD_DIR)/test_ExDoubleList.o: $(TEST_SRC)/ntos_ex/ExDoubleList.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -c -o $@ $<

$(BUILD_DIR)/test_ExSingleList.o: $(TEST_SRC)/ntos_ex/ExSingleList.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -I$(TEST_SRC)/ntos_ex -c -o $@ $<

$(BUILD_DIR)/test_RtlRangeList.o: $(TEST_SRC)/rtl/RtlRangeList.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -c -o $@ $<

$(BUILD_DIR)/test_FsRtlMcb.o: $(TEST_SRC)/ntos_fsrtl/FsRtlMcb.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -c -o $@ $<

$(BUILD_DIR)/test_FsRtlTunnel.o: $(TEST_SRC)/ntos_fsrtl/FsRtlTunnel.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -c -o $@ $<

$(BUILD_DIR)/test_FsRtlRemoveDotsFromPath.o: $(TEST_SRC)/novp_fsrtl/FsRtlRemoveDotsFromPath.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -c -o $@ $<

# -O0: setjmp/longjmp SEH with volatile locals requires unoptimized code
$(BUILD_DIR)/test_RtlException.o: $(TEST_SRC)/rtl/RtlException.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -O0 -c -o $@ $<

$(BUILD_DIR)/test_ExUuid.o: $(TEST_SRC)/ntos_ex/ExUuid.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -c -o $@ $<

# ExInterlocked: -D'__int64=long long' for MSVC type used before our headers
# -Wno-incompatible-pointer-types: LP64 long vs int (LONG) mismatch in extern declarations
$(BUILD_DIR)/test_ExInterlocked.o: $(TEST_SRC)/ntos_ex/ExInterlocked.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -D'__int64=long long' -Wno-incompatible-pointer-types -c -o $@ $<

# --- Kernel Ke implementation objects ---

$(BUILD_DIR)/devqueue.o: $(KE_SRC)/devqueue.c | $(BUILD_DIR)
	$(CC) $(KERNEL_CFLAGS) -c -o $@ $<

$(BUILD_DIR)/test_KeDeviceQueue.o: $(TEST_SRC)/ntos_ke/KeDevQueue.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -c -o $@ $<

$(BUILD_DIR)/test_KeIrql.o: $(TEST_SRC)/ntos_ke/KeIrql.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -c -o $@ $<

# -O0: prevent compiler from optimizing away NULL dereference and guard page access
$(BUILD_DIR)/test_GuardedMemory.o: $(TEST_SRC)/example/GuardedMemory.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -O0 -c -o $@ $<

$(BUILD_DIR)/test_runner.o: test_runner.c | $(BUILD_DIR)
	$(CC) $(TEST_CFLAGS) -c -o $@ $<

clean:
	rm -rf $(BUILD_DIR)
