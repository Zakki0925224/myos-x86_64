# Source files
SRC_FILES := string.c syscalls.c

# Object files
OBJ_FILES := $(SRC_FILES:.c=.o)

# Library file
LIB_FILE := libm.a

# Compiler flags
CC := gcc
CFLAGS := -Wall -g -m64 -nostdlib -fno-builtin -std=c11

# Create static library
$(LIB_FILE): $(OBJ_FILES)
	ar rcs $@ $^
	nm $@

# Compile object files
%.o: %.c
	$(CC) -c $< -o $@ $(CFLAGS)

# Clean rule
clean:
	rm -f $(OBJ_FILES) $(LIB_FILE)

# Default target
all: $(LIB_FILE)

.PHONY: clean all
