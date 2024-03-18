# Source file (C code)
SRC_FILE := libm.c

# Object file
OBJ_FILE := libm.so

# Libary file
LIB_FILE := libm.a

# Compiler flags
CC := gcc
CFLAGS := -Wall -g

# Create static libary
$(LIB_FILE): $(OBJ_FILE)
	ar rcs $@ $<
	nm $@

# Compile static library
$(OBJ_FILE): $(SRC_FILE)
	$(CC) -c $< -o $@ $(CFLAGS)

# Clean rule
clean:
	rm -f $(OBJ_FILE) $(LIB_FILE)

# Default target
all: $(LIB_FILE)

.PHONY: clean all
