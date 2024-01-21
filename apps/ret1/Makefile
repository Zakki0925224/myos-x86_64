# Makefile for compiling and linking C code

# Source file (C code)
SRC_FILE := ret1.c

# Output file (executable)
OUT_FILE := ret1.elf

# Object file
OBJ_FILE := ret1.o

# Compiler flags
CC := gcc
CFLAGS := -O2 -Wall -g -m64 -ffreestanding

# Linker flags
LD := ld.lld
LDFLAGS := --entry main -z norelro --image-base 0 --static

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
