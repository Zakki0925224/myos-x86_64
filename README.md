# myos-x86_64

**myos-x86_64** is a hobby operating system written in Rust.

This is a replacement project for the previous **[myos](https://github.com/Zakki0925224/myos)**.

## Screenshots

![](https://github.com/Zakki0925224/myos-x86_64/assets/49384910/b134ef0a-c94e-46f8-a578-a6e160747fae)
![](https://github.com/Zakki0925224/myos-x86_64/assets/49384910/fce1c2e4-f56b-46fa-8530-9eeec6069591)

## Features

-   [x] Written in Rust
-   [x] My own UEFI boot loader by using [uefi-rs](https://github.com/rust-osdev/uefi-rs)
-   [x] x86_64 kernel
-   [x] PIC (Intel 8259A)
-   [x] Paging
-   Bus support
    -   [x] PCI
    -   [x] USB
-   Device support
    -   [x] PS/2 Keyboard and Mouse
    -   [x] Serial connection (UART 16650A)
    -   [ ] xHCI (WIP)
    -   [ ] VirtIO (WIP)
        -   [ ] virtio-net (WIP)
-   [x] GUI support by using UEFI GOP
-   [x] Initramfs (FAT32 formatted image)
-   [x] My own virtual file system (WIP, read only)
-   [x] [Userland applications](/apps/) (Standard C library for myos available [here](/apps/libm/))
-   [x] Async runtime

## Third party

-   OVMF from [EDK II](https://github.com/tianocore/edk2.git) (included)
-   [Cozette](https://github.com/slavfox/Cozette.git) (download released binary when build)
-   [QEMU](https://gitlab.com/qemu-project/qemu.git) (for debugging)

## How to build

### Minimum packages required to build and run

-   For build kernel

    -   rustup (and Rust toolchain)
    -   python3
    -   build-essential
    -   lld
    -   gcc-multilib
    -   clang
    -   qemu-system
    -   dosfstools

-   For build Cozette

    -   python3-venv
    -   bdf2psf (convert bdf file due to [bug in cozette.psf](https://github.com/slavfox/Cozette/issues/112))

-   For build QEMU

    -   ninja-build
    -   meson
    -   libglib2.0-dev
    -   libsdl2-dev

```bash
$ git clone https://github.com/Zakki0925224/myos-x86_64.git
$ cd myos-x86_64
$ python3 ./task.py task_build
```

## How to run kernel test

```bash
$ cd myos-x86_64/kernel
$ cargo test
```

If you run `task.py` without an argument, you can see the list of commands.
