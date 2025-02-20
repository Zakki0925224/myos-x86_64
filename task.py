import os
import subprocess
import sys

APPS_DIR = "apps"
APPS_LIBC_DIR = "libc"
OUTPUT_DIR = "build"
BOOTLOADER_DIR = "bootloader"
KERNEL_DIR = "kernel"
DUMP_DIR = "dump"
THIRD_PARTY_DIR = "third-party"
QEMU_DIR = "qemu"
DOOM_DIR = "doom-for-myos"
INITRAMFS_DIR = "initramfs"
MNT_DIR_PATH = "/mnt"

BOOTLOADER_FILE = "bootx64.efi"
KERNEL_FILE = "kernel.elf"
IMG_FILE = "myos.img"
ISO_FILE = "myos.iso"
FONT_FILE = "font.psf"
COZETTE_BDF = "cozette.bdf"
OVMF_CODE_FILE = "OVMF_CODE.fd"
QEMU_TRACE_FILE = "qemu_trace"
DOOM_WAD_FILE = "doom1.wad"
INITRAMFS_IMG_FILE = "initramfs.img"

GIT_CHECKOUT_TO_LATEST_TAG = "git fetch --tags && latestTag=$(git describe --tags `git rev-list --tags --max-count=1`) && git checkout $latestTag && git gc"

QEMU_ARCH = "qemu-system-x86_64"
QEMU_TARGET_ARCH = "x86_64-softmmu"

NETDEV_TAP = "tap0"
NETDEV_BR = "br0"
NETDEV_IP = "192.168.100.1/24"

QEMU_DEVICES = [
    # "-device nec-usb-xhci,id=xhci",
    # "-device usb-kbd",
    # "-device virtio-net,netdev=net0,mac=52:54:00:12:34:56 -netdev user,id=net0",
    # f"-device rtl8139,netdev=net0, -netdev user,id=net0,hostfwd=tcp:127.0.0.1:1234-:80 -object filter-dump,id=f0,netdev=net0,file={DUMP_DIR}/dump.pcap",  # <- curl localhost:1234
    "-device ahci,id=ahci",
    "-device ide-cd,drive=disk,bus=ahci.0,bootindex=1",
    "-device isa-debug-exit,iobase=0xf4,iosize=0x04",
    "-audiodev pa,id=speaker -machine pcspk-audiodev=speaker",
    f"-device rtl8139,netdev=net0, -netdev tap,id=net0,ifname={NETDEV_TAP},script=no,downscript=no -object filter-dump,id=f0,netdev=net0,file={DUMP_DIR}/dump.pcap",  # <- ping -4 192.168.100.2
]

QEMU_DRIVES = [
    f"-drive id=disk,if=none,format=raw,file=./{OUTPUT_DIR}/{IMG_FILE}",
    f"-drive if=pflash,format=raw,readonly=on,file=./{THIRD_PARTY_DIR}/{OVMF_CODE_FILE}",
]

QEMU_ARGS = [
    "-accel kvm",
    "-cpu host",
    "-no-reboot",
    "-no-shutdown",
    "-m 512M",
    "-serial mon:stdio",
    "-monitor telnet::5678,server,nowait",
    "-gdb tcp::3333",
]

is_kernel_test = False
test_kernel_path = ""


def qemu_cmd() -> str:
    global is_kernel_test

    qemu_args = " ".join(QEMU_ARGS)
    qemu_drives = " ".join(QEMU_DRIVES)
    qemu_devices = " ".join(QEMU_DEVICES)

    if is_kernel_test:
        qemu_args += " -display none"

    return f"{QEMU_ARCH} {qemu_args} {qemu_drives} {qemu_devices}"


def own_qemu_cmd() -> str:
    return f"./{THIRD_PARTY_DIR}/{QEMU_DIR}/build/{qemu_cmd()} --display sdl --trace events=./{QEMU_TRACE_FILE}"


def git_submodule_update_cmd(path: str) -> str:
    return f"git submodule update --init --recursive {path}"


def run_cmd(
    cmd: str,
    dir: str = "./",
    ignore_error: bool = False,
    check_qemu_exit_code: bool = False,
):
    print(f"\033[32m{cmd}\033[0m")
    cp = subprocess.run(cmd, shell=True, cwd=dir)
    exit_code = cp.returncode
    if check_qemu_exit_code:
        if exit_code == 33:  # EXIT_SUCCESS
            print("Received QEMU exit code: EXIT_SUCCESS")
            exit(0)
        elif exit_code == 35:  # EXIT_FAILURE
            print("Received QEMU exit code: EXIT_FAILURE")
            exit(1)
        else:
            print(f"Received QEMU exit code: Unknown({exit_code})")
            exit(1)

    if exit_code != 0 and not ignore_error:
        exit(exit_code)


# tasks
def init():
    run_cmd(f"mkdir -p ./{OUTPUT_DIR}")


def build_cozette():
    d = f"./{THIRD_PARTY_DIR}"

    if not os.path.exists(f"./{THIRD_PARTY_DIR}/{FONT_FILE}"):
        run_cmd(
            f'wget -qO- https://api.github.com/repos/slavfox/Cozette/releases/latest | grep "{COZETTE_BDF}" | cut -d : -f 2,3 | tr -d \\" | wget -O ./{COZETTE_BDF} -i -',
            dir=d,
            ignore_error=True,
        )
        run_cmd(
            f"bdf2psf --fb ./{COZETTE_BDF} /usr/share/bdf2psf/standard.equivalents /usr/share/bdf2psf/fontsets/Uni2.512 512 ./{FONT_FILE}",
            dir=d,
        )
        run_cmd(f"rm ./{COZETTE_BDF}", dir=d)


def build_qemu():
    global is_kernel_test

    d = f"./{THIRD_PARTY_DIR}/{QEMU_DIR}"
    run_cmd(git_submodule_update_cmd(d))

    if is_kernel_test:
        return

    if not os.path.exists(f"{d}/build/{QEMU_ARCH}"):
        # run_cmd(f"{GIT_CHECKOUT_TO_LATEST_TAG}", dir=d)
        extra_cflags = '--extra-cflags="-DDEBUG_RTL8139"'
        # extra_cflags = ""
        run_cmd(
            f"mkdir -p build && cd build && ../configure --target-list={QEMU_TARGET_ARCH} --enable-trace-backends=log --enable-sdl {extra_cflags} && make -j$(nproc)",
            dir=d,
        )


def build_doom():
    # download doom1.wad
    if not os.path.exists(f"./{THIRD_PARTY_DIR}/{DOOM_WAD_FILE}"):
        run_cmd(
            f"wget -P ./{THIRD_PARTY_DIR} https://distro.ibiblio.org/slitaz/sources/packages/d/doom1.wad"
        )

    d = f"./{THIRD_PARTY_DIR}/{DOOM_DIR}"
    run_cmd(git_submodule_update_cmd(d))
    run_cmd("git checkout master", dir=d)
    run_cmd("make -f Makefile.myos", dir=d)
    # run_cmd("make", dir=d)
    run_cmd(f"cp {d}/doomgeneric ./{APPS_DIR}/doom.elf")
    run_cmd(f"cp ./{THIRD_PARTY_DIR}/{DOOM_WAD_FILE} ./{INITRAMFS_DIR}")


def build_bootloader():
    d = f"./{BOOTLOADER_DIR}"

    init()
    run_cmd("cargo build", d)
    run_cmd(
        f"cp ./target/x86_64-unknown-uefi/debug/bootloader.efi ../{OUTPUT_DIR}/{BOOTLOADER_FILE}",
        d,
    )


def build_kernel():
    global is_kernel_test, test_kernel_path
    d = f"./{KERNEL_DIR}"
    kernel_path = (
        test_kernel_path
        if is_kernel_test and test_kernel_path != ""
        else "./target/x86_64-kernel/debug/kernel"
    )

    init()
    run_cmd("cargo build", d)
    run_cmd(f"cp {kernel_path} ../{OUTPUT_DIR}/{KERNEL_FILE}", d)


def build():
    init()
    build_cozette()
    build_qemu()
    build_bootloader()
    build_kernel()


def build_apps():
    d = f"./{APPS_DIR}"
    dirs = [f for f in os.listdir(d) if os.path.isdir(os.path.join(d, f))]
    dirs.sort()
    dirs.remove(APPS_LIBC_DIR)

    for dir_name in dirs:
        pwd = f"{d}/{dir_name}"

        if os.path.exists(f"{pwd}/Makefile"):
            run_cmd("make clean", dir=pwd)
            run_cmd("make", dir=pwd)

    # copy apps dir to initramfs dir
    run_cmd(f"rm -rf ./{INITRAMFS_DIR}/{APPS_DIR}")
    run_cmd(f"cp -r {d} ./{INITRAMFS_DIR}/")

    # remove `target` directory
    run_cmd(f'find ./{INITRAMFS_DIR} -type d -name "target" | xargs rm -rf')


def make_initramfs():
    global is_kernel_test

    build_doom()

    if not is_kernel_test:
        build_apps()

    run_cmd(
        f"dd if=/dev/zero of=./{OUTPUT_DIR}/{INITRAMFS_IMG_FILE} bs=1M count=128"
    )  # 128MiB
    run_cmd(
        f'mkfs.fat -n "INITRAMFS" -F 32 -s 2 ./{OUTPUT_DIR}/{INITRAMFS_IMG_FILE}'
    )  # format for FAT32
    run_cmd(f"sudo mount -o loop ./{OUTPUT_DIR}/{INITRAMFS_IMG_FILE} {MNT_DIR_PATH}")
    run_cmd(f"sudo rm -rf {MNT_DIR_PATH}/*")  # clear initramfs
    run_cmd(f"sudo cp -r ./{INITRAMFS_DIR}/* {MNT_DIR_PATH}/")
    run_cmd("sleep 0.5")
    run_cmd(f"sudo umount {MNT_DIR_PATH}")


def make_img():
    build()
    make_initramfs()
    run_cmd(f"qemu-img create -f raw ./{OUTPUT_DIR}/{IMG_FILE} 200M")
    run_cmd(
        f'mkfs.fat -n "MYOS" -F 32 -s 2 ./{OUTPUT_DIR}/{IMG_FILE}'
    )  # format for FAT32
    run_cmd(f"sudo mount -o loop ./{OUTPUT_DIR}/{IMG_FILE} {MNT_DIR_PATH}")
    run_cmd(f"sudo mkdir -p {MNT_DIR_PATH}/EFI/BOOT")
    run_cmd(f"sudo mkdir -p {MNT_DIR_PATH}/EFI/myos")
    run_cmd(
        f"sudo cp ./{OUTPUT_DIR}/{BOOTLOADER_FILE} {MNT_DIR_PATH}/EFI/BOOT/BOOTX64.EFI"
    )
    run_cmd(f"sudo cp ./{OUTPUT_DIR}/{KERNEL_FILE} {MNT_DIR_PATH}/EFI/myos/kernel.elf")
    run_cmd(f"sudo cp ./{OUTPUT_DIR}/{INITRAMFS_IMG_FILE} {MNT_DIR_PATH}/initramfs.img")
    run_cmd("sleep 0.5")
    run_cmd(f"sudo umount {MNT_DIR_PATH}")


def make_iso():
    make_img()
    run_cmd(f"dd if=./{OUTPUT_DIR}/{IMG_FILE} of=./{OUTPUT_DIR}/{ISO_FILE} bs=1M")


def make_netdev():
    run_cmd(f"sudo ip link add name {NETDEV_BR} type bridge")
    run_cmd(f"sudo ip addr add {NETDEV_IP} dev {NETDEV_BR}")
    run_cmd(f"sudo ip link set {NETDEV_BR} up")

    run_cmd(f"sudo ip tuntap add {NETDEV_TAP} mode tap")
    run_cmd(f"sudo ip link set {NETDEV_TAP} up")

    run_cmd(f"sudo ip link set {NETDEV_TAP} master {NETDEV_BR}")


def del_netdev():
    run_cmd(f"sudo ip link del {NETDEV_BR}")
    run_cmd(f"sudo ip link del {NETDEV_TAP}")


def run():
    global is_kernel_test

    make_img()
    # cmd = qemu_cmd() if is_kernel_test else own_qemu_cmd()
    cmd = qemu_cmd()

    run_cmd(f"mkdir -p ./{DUMP_DIR}")
    run_cmd(cmd, ignore_error=not is_kernel_test, check_qemu_exit_code=is_kernel_test)


def run_nographic():
    make_img()
    run_cmd(f"{qemu_cmd()} -nographic", ignore_error=True)


def run_with_gdb():
    make_img()
    run_cmd(f"{qemu_cmd()} -S")


def monitor():
    run_cmd("telnet localhost 5678")


def gdb():
    run_cmd(f'rust-gdb ./{OUTPUT_DIR}/{KERNEL_FILE} -ex "target remote :3333"')


def dump():
    build()
    run_cmd(f"mkdir -p ./{DUMP_DIR}")
    run_cmd(f"objdump -d ./{OUTPUT_DIR}/{KERNEL_FILE} > ./{DUMP_DIR}/dump_kernel.txt")
    run_cmd(
        f"objdump -d ./{OUTPUT_DIR}/{BOOTLOADER_FILE} > ./{DUMP_DIR}/dump_bootloader.txt"
    )


def kernel_test_runner(kernel_path: str):
    global is_kernel_test, test_kernel_path
    os.chdir("../")
    is_kernel_test = True
    test_kernel_path = kernel_path
    run()


def clean():
    run_cmd(f"rm -rf ./{OUTPUT_DIR}")
    run_cmd(f"rm -rf ./{DUMP_DIR}")
    run_cmd(f"rm -f ./{THIRD_PARTY_DIR}/{DOOM_WAD_FILE}")
    run_cmd(f"rm -f ./{THIRD_PARTY_DIR}/{FONT_FILE}")
    run_cmd(f"rm -f ./{THIRD_PARTY_DIR}/{COZETTE_BDF}")
    run_cmd(f"rm -rf ./{THIRD_PARTY_DIR}/{DOOM_DIR}/build")
    run_cmd(f"rm -f ./{APPS_DIR}/doom.elf")
    run_cmd(f"rm -rf ./{THIRD_PARTY_DIR}/{QEMU_DIR}/build")
    run_cmd("cargo clean", dir=f"./{BOOTLOADER_DIR}")
    run_cmd("cargo clean", dir="./common")
    run_cmd("cargo clean", dir="./kernel")

    # clean apps
    apps_dir = f"./{APPS_DIR}"
    app_dirs = [
        f for f in os.listdir(apps_dir) if os.path.isdir(os.path.join(apps_dir, f))
    ]

    for dir_name in app_dirs:
        pwd = f"{apps_dir}/{dir_name}"

        if os.path.exists(f"{pwd}/Makefile"):
            run_cmd("make clean", dir=pwd)
        else:
            run_cmd("cargo clean", dir=pwd)


TASKS = [
    init,
    build_cozette,
    build_qemu,
    build_doom,
    build_bootloader,
    build_kernel,
    build,
    build_apps,
    make_initramfs,
    make_img,
    make_iso,
    make_netdev,
    del_netdev,
    run,
    run_nographic,
    run_with_gdb,
    monitor,
    gdb,
    dump,
    clean,
]

if __name__ == "__main__":
    args = sys.argv

    if len(args) >= 2:
        if args[1] == "test" and len(args) >= 3:
            kernel_test_runner(args[2])
            exit(0)

        for task in TASKS:
            if task.__name__ == args[1]:
                task()
                exit(0)

        print("Invalid task name.")
    else:
        print(f"Usage: {list(map(lambda x: x.__name__, TASKS))}")
