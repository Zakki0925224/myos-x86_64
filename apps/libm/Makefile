SRC_FILES := string.c syscalls.c printf.c malloc.c
OBJ_FILES := $(SRC_FILES:.c=.o)
LIB_FILE := libm.a

CC := gcc
CFLAGS := -Wall -g -m64 -nostdlib -fno-builtin -mno-sse -mno-mmx -std=c11

$(LIB_FILE): $(OBJ_FILES)
	ar rcs $@ $^
	nm $@

%.o: %.c
	$(CC) -c $< -o $@ $(CFLAGS)

clean:
	rm -f $(OBJ_FILES) $(LIB_FILE)

all: $(LIB_FILE)

.PHONY: clean all
