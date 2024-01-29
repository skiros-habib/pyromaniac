#! /bin/bash

# script based off https://github.com/firecracker-microvm/firecracker/blob/49db07b32df5cc0dde0ff4a9e04a431f17340091/resources/rebuild.sh#L168

set -eu -o pipefail

set -x
PS4='+\t '

ver="$1"

# check if command line argument is there
if [ -z "$1" ]; then
    echo "provide version for which to build kernel"
    exit 0
fi

allowed=("4.14" "5.10" "6.1")

if [[ ! " ${allowed[@]} " =~ " ${ver} " ]]; then
    echo "unrecognised version $1"
    exit 0
fi

prev=$(pwd)

mkdir -p /tmp/linux
cd /tmp/linux

echo "Downloading the latest patch version for v$ver..."
major_version="${ver%%.*}"
url_base="https://cdn.kernel.org/pub/linux/kernel"
LATEST_VERSION=$(
    curl -fsSL $url_base/v$major_version.x/ \
    | grep -o "linux-$ver\.[0-9]*\.tar.xz" \
    | sort -rV \
    | head -n 1 || true)
# Fetch tarball and sha256 checksum.
curl -fsSLO "$url_base/v$major_version.x/sha256sums.asc"
curl -fsSLO "$url_base/v$major_version.x/$LATEST_VERSION"
# Verify checksum.
grep "${LATEST_VERSION}" sha256sums.asc | sha256sum -c -
echo "Extracting the kernel source..."
tar -xaf $LATEST_VERSION
DIR=$(basename $LATEST_VERSION .tar.xz)

cd $DIR
pushd .

arch=$(uname -m)
if [ "$arch" = "x86_64" ]; then
    format="elf"
    target="vmlinux"
    binary_path="$target"
elif [ "$arch" = "aarch64" ]; then
    format="pe"
    target="Image"
    binary_path="arch/arm64/boot/$target"
else
    echo "FATAL: Unsupported architecture!"
    exit 1
fi

cp $prev/firecracker/resources/guest_configs/microvm-kernel-ci-x86_64-$ver.config .config

make olddefconfig
make -j $(nproc) $target
LATEST_VERSION=$(cat include/config/kernel.release)
flavour=$(basename microvm-kernel-ci-x86_64-$ver.config .config |grep -Po "\d+\.\d+\K(-.*)" || true)

cp -v $binary_path $prev/resources/kernel.bin

popd &>/dev/null

rm -rf /tmp/linux