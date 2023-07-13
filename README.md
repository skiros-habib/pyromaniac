# Pyromaniac
![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/joeyh021/pyromaniac/ci.yml?label=CI)


_Remote Code Execution as a Service_

A system for secure and high-performance execution of arbitrary code, powered by [Firecracker microVMs](https://github.com/firecracker-microvm/firecracker)

## Getting Started

Make sure the machine you intend to use supports KVM, and that you have access to it. See [the Firecracker docs](https://github.com/firecracker-microvm/firecracker/blob/main/docs/getting-started.md) for instructions on how to do this. You'll need [a Rust toolchain](https://rustup.rs/) and [Docker](https://docs.docker.com/engine/install/).

Pyromaniac looks for the VM resources in a `resources` directory. Put the firecracker binary there, either download the [latest release from Github](https://github.com/firecracker-microvm/firecracker/releases/latest), or clone the firecracker repo and compile your own copy:

```
git clone https://github.com/firecracker-microvm/firecracker
./firecracker/tools/devtool build
cp ./firecracker/build/cargo_target/x86_64-unknown-linux-musl/release/firecracker ./resources
```

You'll also need a kernel build, and a rootfs with `pyrod` in it.

See [Firecracker docs](https://github.com/firecracker-microvm/firecracker/blob/main/docs/rootfs-and-kernel-setup.md) for full details.

### Kernel

Specific kernel configs are required to include the device drivers needed for firecracker. The easiest thing to do is to build in Firecracker's devcontainer using their provided configs. The commands below are for v5.10 on x86_64. The output will be under `firecracker/build/kernel/linux-5.10`

```sh
git clone https://github.com/firecracker-microvm/firecracker
cd firecracker
./tools/devtool build_kernel -c resources/guest_configs/microvm-kernel-x86_64-5.10.config -n $(nproc)
cp build/kernel/linux-5.10/vmlinux-5.10-x86_64.bin ../resources/kernel.bin
```

### RootFS

Different images are used for different languages. From the `pyromaniac` root directory, run:

```
./scripts/mkrootfs.sh <language>
```

This will
1. Build `pyrod` 
    - `pyrod` is built for `x86_64-unknown-linux-musl`, you'll have to install that target via rustup
2. Create a new image file and mount it
3. Build an alpine-based Docker container and copy it's root filesystem into the image file
4. Copy the image file into the resources directory

### Starting the Server

Copy `.env.example` to `.env` and configure it with your desired port, and the **full path** to the resource directory.

```
cargo run --bin=pyromaniac
```

This will launch the server on the given port. For other config options for the server, such as configuring the firecracker runtime options see `pyromaniac/src/config.rs`. When running in debug mode, jailer is not used, and the console output from the VM is written to `vm.out`.

The server exposes a single endpoint, `/api/run`, which accepts JSON with the following schema:

```json
{
    "lang": "Python",
    "code": "print(f\"Hello, {}!\")",
    "input": "joeyh021"
}
```

The response will look like:

```json
{
    "stdout": "Hello, joeyh021",
    "stderr": "",
}
```

## Deployment in Production

You'll need a firecracker binary and kernel and rootfs as before, but you'll also need a jailer binary, and to take a few extra steps to secure the machine you're running on. A jailer binary can be built the same as firecracker (details above), and can be found at `firecracker/build/cargo_target/x86_64-unknown-linux-musl/release`. Place this next to the firecracker binary.

For jailer to run securely, it needs a dedicated system user with no privileges to that the firecracker process will run as:

```
addgroup --system --gid 222 jail
adduser --system --no-create-home --shell /bin/false --disabled-login --gid 222 --uid 222 jail
```

Make sure to set the uid and gid in `.env`. The server will by default use jailer instead of just firecracker when compiled with `--release`.

Jailer uses certain kernel features to drop privileges for the firecracker binary, which requires that jailer run as root. Start the server with

```
cargo build --release --bin=pyromaniac 
sudo target/release/pyromaniac
```

While running the server as root is not ideal, this does not affect the security of firecracker or jailer's sandboxing. See [this issue](https://github.com/firecracker-microvm/firecracker/issues/1190) for further discussion.

For full security recommendations see https://github.com/firecracker-microvm/firecracker/blob/main/docs/prod-host-setup.md

## Development

### Adding a new language

Currently supported languages:
- Python
- Rust
- Java

See [here](docs/languages.md) for full info on the details of each supported language.

To add a new one:
- Add a new implemenation of the `Runner` trait in `pyrod/src/run`
    - `Runner::compile` should write the code out to a file, and then run the compiler if necessary
    - `Runner::run` should execute the compiled file
- Add it to the `Language` enum (and it's `impl`s) in `pyrod/src/run/mod.rs`
- Add a new rootfs build for it by creating a new Dockerfile in `scripts/images`
