# Compiler
ifeq ($(OS),Windows_NT)
    CC = clang
    CFLAGS = -Iinclude -Wall -Werror -O2 -fPIC
else
	CC ?= clang
    CFLAGS = -Iinclude -Wall -O2 -fPIC
endif

ifeq ($(CC),clang)
    CFLAGS += -ferror-limit=0
endif

# Append sysroot if defined (according to Cross.toml)
ifdef SYSROOT
    CFLAGS += --sysroot=$(SYSROOT)
endif

# Append target if defined (according to Cross.toml)
ifdef CLANG_TARGET
	CFLAGS += -target $(CLANG_TARGET)
endif


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
ARCH ?= $(shell uname -m)
# Additional architecture detection and setting
ifneq (,$(findstring aarch64,$(CC)))
  ARCH = aarch64
else ifneq (,$(findstring arm64,$(ARCH)))
  ARCH = aarch64
else ifneq (,$(findstring iPhone,$(ARCH)))
  ARCH = aarch64
  CFLAGS += -DHAVE_MALLOC_MALLOC
else ifneq (,$(findstring powerpc64le,$(CC)))
  ARCH = ppc64le
endif


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

# Clean rule
.PHONY: clean
clean:
	rm -f $(SRCDIR)/*.o $(TARGET)
