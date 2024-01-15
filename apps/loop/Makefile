# Makefile for assembling and linking x86_64 assembly code

# Source file (assembly code)
SRC_FILE := loop.S

# Output file (executable)
OUT_FILE := loop.elf

# Object file
OBJ_FILE := loop.o

# NASM flags
NASM_FLAGS := -f elf64

# Assemble and link
$(OUT_FILE): $(OBJ_FILE)
	ld $< -o $(OUT_FILE)

# Assemble
$(OBJ_FILE): $(SRC_FILE)
	nasm $(NASM_FLAGS) $< -o $(OBJ_FILE)

# Clean rule
clean:
	rm -f $(OUT_FILE) $(OBJ_FILE)

# Default target
all: $(OUT_FILE)

.PHONY: clean all
