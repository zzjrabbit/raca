# How to run the project

## Using a pre-built image
You can download a image built by github actions at [the release page](https://github.com/zzjrabbit/raca/releases). \
Then you need to install [qemu](https://www.qemu.org/download/). \
And [ovmf](https://github.com/osdev0/edk2-ovmf-nightly/releases/tag/nightly-20260214T015320Z) is also required. \
After downloading the archieve, extract ovmf-code-loongarch64.fd. \
Finally, you can launch the kernel by the following command.
``` sh
qemu-system-loongarch64 \
    -machine virt \
    -m 512 m \
    -smp cores=4 \
    -cpu la464 \
    # Add if you need to use Hyper-V acceleration.
    # -accel whpx \
    -serial stdio \
    -device nvme,drive=disk,serial=deadbeef \
    -drive if=none,format=raw,id=disk,file=PATH_TO_THE_IMAGE \
    -drive if=pflash,format=raw,file=PATH_TO_OVMF \
    -device ramfb
```

## Build an image and run it.
This requires less work of human, while your computer might smoke. \
You only need to run the following command at the root of the repository.
``` sh
cargo krun -s
```
