# myos-x86_64

**myos-x86_64** is a hobby operating system written in Rust.

This is a replacement project for the previous **[myos](https://github.com/Zakki0925224/myos)**.

## Screenshots

![](https://github.com/user-attachments/assets/7cc7d545-b3ca-4042-b145-73a909834c13)

### Old

![](https://github.com/Zakki0925224/myos-x86_64/assets/49384910/b134ef0a-c94e-46f8-a578-a6e160747fae)
![](https://github.com/Zakki0925224/myos-x86_64/assets/49384910/fce1c2e4-f56b-46fa-8530-9eeec6069591)

### doomgeneric

[![doomgeneric - myos-x86_64](http://img.youtube.com/vi/DRtx9h6xlkg/0.jpg)](https://www.youtube.com/watch?v=DRtx9h6xlkg)

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
    -   [x] UART 16650A
    -   [x] RTL8139
    -   [ ] xHCI (WIP)
    -   [ ] VirtIO (WIP)
        -   [ ] virtio-net (WIP)
-   [x] File system
    -   [x] Own VFS
    -   [x] FAT32 (read only)
-   [x] Networking
    -   [x] ARP
    -   [ ] IPv4 (ICMP, TCP) (WIP)
-   [x] GUI support by using UEFI GOP
-   [x] [Userland applications](/apps/) (libc for myos available [here](/apps/libc/))
-   [x] Async runtime
-   [x] DOOM challenge!

## Third party

-   OVMF from [EDK II](https://github.com/tianocore/edk2.git) (included)
-   [Cozette](https://github.com/slavfox/Cozette.git) (download released binary when build)
-   [QEMU](https://gitlab.com/qemu-project/qemu.git) (for debugging)
-   [doom-for-myos](https://github.com/Zakki0925224/doom-for-myos) (forked from [ozkl/doomgeneric](https://github.com/ozkl/doomgeneric))
-   [doom1.wad](https://distro.ibiblio.org/slitaz/sources/packages/d/doom1.wad)

## How to run

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
    -   wget

-   For build Cozette

    -   python3-venv
    -   bdf2psf (convert bdf file due to [bug in cozette.psf](https://github.com/slavfox/Cozette/issues/112))

-   For build QEMU

    -   ninja-build
    -   meson
    -   libglib2.0-dev
    -   libsdl2-dev

```bash
# install required packages
$ sudo apt update && sudo apt install python3 build-essential lld gcc-multilib clang qemu-system dosfstools wget python3-venv bdf2psf ninja-build meson libglib2.0-dev libsdl2-dev

$ git clone https://github.com/Zakki0925224/myos-x86_64.git
$ cd myos-x86_64
$ python3 ./task.py run
```

## How to run kernel test

```bash
$ cd myos-x86_64/kernel
$ cargo test
```

If you run `task.py` without an argument, you can see the list of commands.
