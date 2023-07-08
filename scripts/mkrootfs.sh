# pass dockerfile as $1
dockerfile="$1"

# check if command line argument is there
if [ -z $1 ]; then
    echo "provide language for which to build rootfs"
    exit 0
fi

# build pyrod
cargo build --release --bin=pyrod --target=x86_64-unknown-linux-musl

# 100MB rootfs - change if gonna run out of space
dd if=/dev/zero of=rootfs.ext4 bs=1M count=100
mkfs.ext4 rootfs.ext4


sudo rm -rf /tmp/rootfs && mkdir /tmp/rootfs
sudo mount rootfs.ext4 /tmp/rootfs

sudo docker build --no-cache . -f scripts/images/Dockerfile.$1 -t pyro-$1
sudo docker run -it --rm -v /tmp/rootfs:/rootfs pyro-$1

sudo umount /tmp/rootfs
mv rootfs.ext4 resources/rootfs-$1.ext4