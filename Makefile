# Compiler
CC ?= gcc

# Compiler flags
CFLAGS = -Iinclude -Wall -Wextra -O2 -fstrict-aliasing -fPIC

# Directories
INCDIR = include
SRCDIR = src

# Source files
SRCS = $(wildcard $(SRCDIR)/*.c)

# Object files
OBJS = $(SRCS:.c=.o)

# Target library
TARGET = libic.a

# Vectorization flags
SSE_FLAGS = -msse4.1
AVX2_FLAGS = -mavx2

# Determine architecture and set vectorization flags
ARCH := $(shell uname -m)
ifeq ($(ARCH),x86_64)
	CFLAGS += $(SSE_FLAGS)
	ifeq ($(AVX2),1)
		CFLAGS += $(AVX2_FLAGS)
	endif
else ifeq ($(ARCH),aarch64)
	CFLAGS += -march=armv8-a
endif

# Rule to create the library
$(TARGET): $(OBJS)
	ar rc $@ $+

# Rule to compile source files into object files
$(SRCDIR)/%.o: $(SRCDIR)/%.c
	$(CC) $(CFLAGS) -c $< -o $@


# Test program
TEST_SRCS = tests.c
TEST_OBJS = $(TEST_SRCS:.c=.o)
TEST_TARGET = test_libic

# Rule to create the test executable
$(TEST_TARGET): $(TEST_OBJS) $(TARGET)
	$(CC) $(CFLAGS) -L. -lic -lm -o $@ $^

# Rule to compile test source files into object files
%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

# Run the test
.PHONY: test
test: $(TEST_TARGET)
	./$(TEST_TARGET)

# Clean rule
.PHONY: clean
clean:
	rm -f $(SRCDIR)/*.o $(TARGET)
