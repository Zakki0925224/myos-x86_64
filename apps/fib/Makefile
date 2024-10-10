SRC_FILES := main.c
OBJ_FILES := $(SRC_FILES:.c=.o)
OUT_FILE := ../fib.elf

CC := gcc
CFLAGS := -O2 -Wall -g -m64 -nostdlib -fno-builtin -mno-sse -mno-mmx -std=c11

LD := ld.lld
LDFLAGS := -z norelro --static

LIB_DIR := ../libm
LIB := -L$(LIB_DIR) -lm

$(OUT_FILE): $(OBJ_FILES)
	make -C $(LIB_DIR)
	$(LD) $(OBJ_FILES) $(LIB) -o $@ $(LDFLAGS)

%.o: %.c
	$(CC) -c $< -o $@ $(CFLAGS)

clean:
	rm -f $(OUT_FILE) $(OBJ_FILES)

all: $(OUT_FILE)

.PHONY: clean all
