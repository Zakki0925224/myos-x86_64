OUT_FILE := hello-rs
TARGET := x86_64-app

$(OUT_FILE):
	cargo build
	cp target/$(TARGET)/debug/$(OUT_FILE) ./$(OUT_FILE).elf

clean:
	rm -f $(OUT_FILE)

all: $(OUT_FILE)

.PHONY: clean all
