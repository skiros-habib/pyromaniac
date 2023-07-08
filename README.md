# Pyromaniac
_Remote Code Execution as a Service (RCEaaS)_

A system for secure and high-performance execution of arbitrary code, powered by [Firecracker microVMs](https://github.com/firecracker-microvm/firecracker)

## Getting Started

You'll need a firecracker and jailer binary places in the `resources` directory. Either download the [latest release from Github](https://github.com/firecracker-microvm/firecracker/releases/latest), or clone firecracker and compile your own copy (note that the firecracker devtool uses Docker):

```
git clone https://github.com/firecracker-microvm/firecracker
./firecracker/tools/devtool build
cp ./firecracker/build/cargo_target/x86_64-unknown-linux-musl/release/firecracker ./resources
cp ./firecracker/build/cargo_target/x86_64-unknown-linux-musl/release/jailer ./resources
```

You'll also need a kernel binary, and a rootfs with `pyrod` in it. See below for instructions on how to build those.

## Building a Kernel and RootFS

See [Firecracker docs](https://github.com/firecracker-microvm/firecracker/blob/main/docs/rootfs-and-kernel-setup.md) for full details.

### Kernel

Specific kernel configs are required to include the device drivers needed for firecracker. The easiest thing to do is to build in Firecracker's devcontainer using their provided configs. The commands below are for v5.10 on x86_64. The output will be under `firecracker/build/kernel/linux-5.10`


```sh
git clone https://github.com/firecracker-microvm/firecracker
./firecracker/tools/devtool build_kernel -c "firecracker/resources/guest_configs/microvm-kernel-x86_64-5.10.config" 
cp firecracker/build/kernel/linux-5.10/vmlinux-5.10-x86_64.bin resources/kernel.bin
```

### RootFS

Different images are needed for different languages. From the `pyromaniac` root directory, run:

```
scripts/mkrootfs.sh <language>
```

This will
1. Build `pyrod`
2. Create a new image file and mount it
3. Build an alpine-based Docker container and copy it's root filesystem into the image file
