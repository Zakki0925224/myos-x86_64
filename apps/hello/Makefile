SRC_FILES := hello.c
OBJ_FILES := $(SRC_FILES:.c=.o)
OUT_FILE := ../hello.elf

CC := gcc
CFLAGS := -I ../libc -O2 -Werror -g -m64 -nostdlib -fno-builtin -mno-sse -mno-mmx -std=c11

LD := ld.lld
LDFLAGS := -z norelro --static

LIB_DIR := ../libc
LIB := -L$(LIB_DIR) -lc

$(OUT_FILE): $(OBJ_FILES)
	make -C $(LIB_DIR)
	$(LD) $(OBJ_FILES) $(LIB) -o $@ $(LDFLAGS)

%.o: %.c
	$(CC) -c $< -o $@ $(CFLAGS)

clean:
	rm -f $(OUT_FILE) $(OBJ_FILES)

all: $(OUT_FILE)

.PHONY: clean all
