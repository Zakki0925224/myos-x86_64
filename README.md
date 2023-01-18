# myos-x86_64

[myos](https://github.com/Zakki0925224/myos) の x86_64・UEFI 版自作 OS。

## Third party

### UEFI firmware

-   [EDK II](https://github.com/tianocore/edk2) (Using forked source)

### Console font

-   [Cozette](https://github.com/slavfox/Cozette) (Using forked source)

## Features

-   [x] Written in Rust and Assembly
-   [x] My own UEFI boot loader by using [uefi-rs](https://github.com/rust-osdev/uefi-rs)
-   [x] x86_64 kernel
-   Device support
    -   [x] Serial connection (UART 16650A)
-   [x] GUI support by using Graphics Output Protocol

## How to build

### Dependent tools

-   TODO

```bash
$ git clone https://github.com/Zakki0925224/myos-x86_64.git --recursive
$ task build
```
