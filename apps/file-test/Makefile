# Source file (C code)
SRC_FILE := file-test.c

# Output file (executable)
OUT_FILE := file-test.elf

# Object file
OBJ_FILE := file-test.o

# Compiler flags
CC := gcc
CFLAGS := -O2 -Wall -g -m64 -nostdlib -fno-builtin -std=c11

# Linker flags
LD := ld.lld
LDFLAGS := -z norelro --static

# Library flags
LIB_DIR := ../libm
LIB := -L$(LIB_DIR) -lm

# Compile and link
$(OUT_FILE): $(OBJ_FILE)
	$(LD) $< $(LIB) -o $@ $(LDFLAGS)

# Compile
$(OBJ_FILE): $(SRC_FILE)
	make -C $(LIB_DIR)
	$(CC) $< $(LIB) -c -o $@ $(CFLAGS)

# Clean rule
clean:
	rm -f $(OUT_FILE) $(OBJ_FILE)

# Default target
all: $(OUT_FILE)

.PHONY: clean all
