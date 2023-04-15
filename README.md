# myos-x86_64

**myos-x86_64** is a hobby operating system written in Rust.

This is a replacement project for the previous **[myos](https://github.com/Zakki0925224/myos)**.

## Third party

-   [EDK II](https://github.com/tianocore/edk2.git)
-   [Cozette](https://github.com/slavfox/Cozette.git)
-   [QEMU](https://gitlab.com/qemu-project/qemu.git) (for debugging)

## Features

-   [x] Written in Rust
-   [x] My own UEFI boot loader by using [uefi-rs](https://github.com/rust-osdev/uefi-rs)
-   [x] x86_64 kernel
-   Device support
    -   [x] Serial connection (UART 16650A)
    -   [x] PCI devices
    -   [x] USB devices (xHC) (work in progress...)
-   [x] GUI support by using Graphics Output Protocol

## How to build

### Dependent tools

#### myos

-   [rust (nightly)]()
-   [go-task](https://github.com/go-task/task)

#### cozette

-   [python3, pip](https://www.python.org/)
-   [pipenv](https://pypi.org/project/pipenv/)
-   [fontforge](https://github.com/fontforge/fontforge)
-   bdf2psf

#### edk2

-   uuid-dev
-   nasm

```bash
$ git clone https://github.com/Zakki0925224/myos-x86_64.git --recursive
$ task build
```
