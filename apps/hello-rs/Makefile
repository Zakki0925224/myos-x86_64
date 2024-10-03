FILE_NAME := hello-rs
OUT_FILE := ../$(FILE_NAME).elf
TARGET := x86_64-app

$(OUT_FILE):
	cargo build
	cp target/$(TARGET)/debug/$(FILE_NAME) $(OUT_FILE)

clean:
	rm -f $(OUT_FILE)

all: $(OUT_FILE)

.PHONY: clean all
