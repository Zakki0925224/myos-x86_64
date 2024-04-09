# Source file (C code)
SRC_FILE := loop.c

# Output file (executable)
OUT_FILE := loop.elf

# Object file
OBJ_FILE := loop.o

# Compiler flags
CC := gcc
CFLAGS := -O2 -Wall -g -m64 -nostdlib -fno-builtin -std=c11

# Linker flags
LD := ld.lld
LDFLAGS := -z norelro --static

# Compile and link
$(OUT_FILE): $(OBJ_FILE)
	$(LD) $< -o $(OUT_FILE) $(LDFLAGS)

# Compile
$(OBJ_FILE): $(SRC_FILE)
	$(CC) -c $< -o $(OBJ_FILE) $(CFLAGS)

# Clean rule
clean:
	rm -f $(OUT_FILE) $(OBJ_FILE)

# Default target
all: $(OUT_FILE)

.PHONY: clean all
