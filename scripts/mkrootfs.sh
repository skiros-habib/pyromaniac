#! /bin/bash
# pass dockerfile as $1
lang="$1"

# check if command line argument is there
if [ -z "$1" ]; then
    echo "provide language for which to build rootfs"
    exit 0
fi

# 100MB rootfs - change if gonna run out of space

case $lang in
"rust")
    size=1000
    ;;
"python")
    size=100
    ;;
"java")
    size=400
    ;;
"bash")
    size=50
    ;;
"sh")
    size=20
    ;;
*)
    echo "unrecognised language"
    exit 0
    ;;
esac


# build pyrod
cargo build --release --bin=pyrod --target=x86_64-unknown-linux-musl

dd if=/dev/zero of=rootfs.ext4 bs=1M count=$size
/usr/sbin/mkfs.ext4 rootfs.ext4


sudo rm -rf /tmp/rootfs && mkdir /tmp/rootfs
sudo mount rootfs.ext4 /tmp/rootfs

sudo docker build --no-cache . -f "scripts/images/Dockerfile.$lang" -t "pyro-$lang"
sudo docker run -it --rm -v /tmp/rootfs:/rootfs "pyro-$lang"

sudo umount /tmp/rootfs
mv rootfs.ext4 "resources/rootfs-$lang.ext4"