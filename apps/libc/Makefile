SRC_FILES := string.c syscalls.c printf.c malloc.c window.c main.c exit.c abs.c temp.c math.c ctype.c
OBJ_FILES := $(SRC_FILES:.c=.o)
LIB_FILE := libc.a

CC := gcc
CFLAGS := -O3 -Werror -g -m64 -nostdlib -fno-builtin -std=c11

$(LIB_FILE): $(OBJ_FILES)
	ar rcs $@ $^
	nm $@

%.o: %.c
	$(CC) -c $< -o $@ $(CFLAGS)

clean:
	rm -f $(OBJ_FILES) $(LIB_FILE)

all: $(LIB_FILE)

.PHONY: clean all
