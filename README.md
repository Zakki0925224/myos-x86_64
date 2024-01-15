# myos-x86_64

**myos-x86_64** is a hobby operating system written in Rust.

This is a replacement project for the previous **[myos](https://github.com/Zakki0925224/myos)**.

## Features

-   [x] Written in Rust
-   [x] My own UEFI boot loader by using [uefi-rs](https://github.com/rust-osdev/uefi-rs)
-   [x] x86_64 kernel
-   [x] Programmable Interrupt Controller (Intel 8259A)
-   Device support
    -   [x] PS/2 Keyboard
    -   [x] Serial connection (UART 16650A)
    -   [x] PCI devices
    -   [ ] USB devices (xHC) (WIP)
-   [x] GUI support by using UEFI GOP
-   [x] Initramfs (but here we call FAT32 formatted image initramfs)

## Third party

-   OVMF from [EDK II](https://github.com/tianocore/edk2.git) (included)
-   [Cozette](https://github.com/slavfox/Cozette.git)
-   [QEMU](https://gitlab.com/qemu-project/qemu.git) (for debugging)

## How to build

### Minimum packages required to build

-   Rust (nightly)
-   Python3
-   dosfstools
-   QEMU
-   make, ld, nasm
-   Some packages for build third party tools

```bash
$ git clone https://github.com/Zakki0925224/myos-x86_64.git
$ cd myos-x86_64
$ python3 ./task.py task_build
```

If you run `task.py` without an argument, you can see the list of commands.
