import os
import subprocess
import sys

APPS_DIR = "apps"
OUTPUT_DIR = "build"
BOOTLOADER_DIR = "bootloader"
KERNEL_DIR = "kernel"
DUMP_DIR = "dump"
THIRD_PARTY_DIR = "third-party"
COZETTE_DIR = "cozette"
QEMU_DIR = "qemu"
INITRAMFS_DIR = "initramfs"
MNT_DIR_PATH = "/mnt"

BOOTLOADER_FILE = "bootx64.efi"
KERNEL_FILE = "kernel.elf"
IMG_FILE = "myos.img"
ISO_FILE = "myos.iso"
COZETTE_FILE = "cozette.psf"
OVMF_CODE_FILE = "OVMF_CODE.fd"
QEMU_TRACE_FILE = "qemu_trace"
INITRAMFS_IMG_FILE = "initramfs.img"

GIT_SUBMODULE_UPDATE = "git submodule update --init --recursive"
GIT_CHECKOUT_TO_LATEST_TAG = "git fetch --tags && latestTag=$(git describe --tags `git rev-list --tags --max-count=1`) && git checkout $latestTag && git gc"

QEMU_ARCH = "qemu-system-x86_64"
QEMU_TARGET_ARCH = "x86_64-softmmu"

QEMU_DEVICES = [
    "-device nec-usb-xhci,id=xhci",
    "-device ahci,id=ahci",
    "-device ide-cd,drive=disk,bus=ahci.0,bootindex=1",
    "-device usb-kbd",
    "-device isa-debug-exit,iobase=0xf4,iosize=0x04",
]

QEMU_DRIVES = [
    f"-drive id=disk,if=none,format=raw,file=./{OUTPUT_DIR}/{IMG_FILE}",
    f"-drive if=pflash,format=raw,readonly=on,file=./{THIRD_PARTY_DIR}/{OVMF_CODE_FILE}",
]

QEMU_ARGS = [
    "-accel kvm",
    "-no-reboot",
    "-no-shutdown",
    "-m 4G",
    "-serial mon:stdio",
    "-monitor telnet::5678,server,nowait",
    "-gdb tcp::3333",
]


def qemu_cmd() -> str:
    qemu_args = " ".join(QEMU_ARGS)
    qemu_drives = " ".join(QEMU_DRIVES)
    qemu_devices = " ".join(QEMU_DEVICES)

    return f"{QEMU_ARCH} {qemu_args} {qemu_drives} {qemu_devices}"


def own_qemu_cmd() -> str:
    return f"./{THIRD_PARTY_DIR}/{QEMU_DIR}/build/{QEMU_TARGET_ARCH}/{qemu_cmd()} --display sdl --trace events=./{QEMU_TRACE_FILE}"


def run_cmd(cmd: str, dir: str = "./", ignore_error: bool = False):
    print(f"\033[32m{cmd}\033[0m")
    cp = subprocess.run(cmd, shell=True, cwd=dir)

    if cp.returncode != 0 and not ignore_error:
        print(f"returncode: {cp.returncode}")
        exit(0)


# tasks
def task_clear():
    run_cmd(f"rm -rf ./{OUTPUT_DIR}")
    run_cmd(f"rm -rf ./{DUMP_DIR}")


def task_init():
    run_cmd(GIT_SUBMODULE_UPDATE)
    run_cmd(f"mkdir -p ./{OUTPUT_DIR}")


def task_build_cozette():
    d = f"./{THIRD_PARTY_DIR}/{COZETTE_DIR}"

    if not os.path.exists(f"./{THIRD_PARTY_DIR}/{COZETTE_FILE}"):
        run_cmd(f"{GIT_CHECKOUT_TO_LATEST_TAG}", dir=d)
        run_cmd(
            "pipenv install --python 3 && pipenv run python3 ./build.py fonts", dir=d
        )
        run_cmd(
            f"bdf2psf --fb ./build/cozette.bdf /usr/share/bdf2psf/standard.equivalents /usr/share/bdf2psf/fontsets/Uni2.512 512 ../{COZETTE_FILE}",
            dir=d,
        )


def task_build_qemu():
    d = f"./{THIRD_PARTY_DIR}/{QEMU_DIR}"

    if not os.path.exists(f"{d}/build/{QEMU_ARCH}"):
        run_cmd(f"{GIT_CHECKOUT_TO_LATEST_TAG}", dir=d)
        run_cmd(
            f'mkdir -p build && cd build && ../configure --target-list={QEMU_TARGET_ARCH} --enable-trace-backends=log --enable-sdl --extra-cflags="-DDEBUG_XHCI" && make -j$(nproc)',
            dir=d,
        )


def task_build_bootloader():
    d = f"./{BOOTLOADER_DIR}"

    task_init()
    run_cmd("cargo build", d)
    run_cmd(
        f"cp ./target/x86_64-unknown-uefi/debug/bootloader.efi ../{OUTPUT_DIR}/{BOOTLOADER_FILE}",
        d,
    )


def task_build_kernel():
    d = f"./{KERNEL_DIR}"

    task_init()
    run_cmd("cargo build", d)
    run_cmd(f"cp ./target/x86_64/debug/kernel ../{OUTPUT_DIR}/{KERNEL_FILE}", d)


def task_build():
    task_clear()
    task_init()
    task_build_cozette()
    task_build_qemu()
    task_build_bootloader()
    task_build_kernel()


def task_build_apps():
    d = f"./{APPS_DIR}"
    dirs = [f for f in os.listdir(d) if os.path.isdir(os.path.join(d, f))]

    for dir_name in dirs:
        pwd = f"{d}/{dir_name}"

        if os.path.exists(f"{pwd}/Makefile"):
            run_cmd("make clean", dir=pwd)
            run_cmd("make", dir=pwd)

    # copy apps dir to initramfs dir
    run_cmd(f"cp -r {d} ./{INITRAMFS_DIR}/")

    # remove `target` directory
    run_cmd(f'find ./{INITRAMFS_DIR} -type d -name "target" | xargs rm -rf')


def task_make_initramfs():
    task_build_apps()

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


def task_make_img():
    task_build()
    task_make_initramfs()
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


def task_make_iso():
    task_make_img()
    run_cmd(f"dd if=./{OUTPUT_DIR}/{IMG_FILE} of=./{OUTPUT_DIR}/{ISO_FILE}")


def task_run():
    task_make_img()
    run_cmd(own_qemu_cmd(), ignore_error=True)


def task_run_nographic():
    task_make_img()
    run_cmd(f"{own_qemu_cmd()} -nographic", ignore_error=True)


def task_run_with_gdb():
    task_make_img()
    run_cmd(f"{own_qemu_cmd()} -S")


def task_monitor():
    run_cmd("telnet localhost 5678")


def task_gdb():
    run_cmd(f'rust-gdb ./{OUTPUT_DIR}/{KERNEL_FILE} -ex "target remote :3333"')


def task_dump():
    task_build()
    run_cmd(f"mkdir -p ./{DUMP_DIR}")
    run_cmd(f"objdump -d ./{OUTPUT_DIR}/{KERNEL_FILE} > ./{DUMP_DIR}/dump_kernel.txt")
    run_cmd(
        f"objdump -d ./{OUTPUT_DIR}/{BOOTLOADER_FILE} > ./{DUMP_DIR}/dump_bootloader.txt"
    )


TASKS = [
    task_clear,
    task_init,
    task_build_cozette,
    task_build_qemu,
    task_build_bootloader,
    task_build_kernel,
    task_build,
    task_build_apps,
    task_make_initramfs,
    task_make_img,
    task_make_iso,
    task_run,
    task_run_nographic,
    task_run_with_gdb,
    task_monitor,
    task_gdb,
    task_dump,
]

if __name__ == "__main__":
    args = sys.argv

    if len(args) == 2:
        for task in TASKS:
            if task.__name__ == args[1]:
                task()
                exit(0)

        print("Invalid task name.")
    else:
        print(f"Usage: {list(map(lambda x: x.__name__, TASKS))}")
