# based on instructions from https://github.com/firecracker-microvm/firecracker/blob/main/docs/rootfs-and-kernel-setup.md
from alpine:3.18

# create service user for untrusted processes to run under
# system group, gid 111
RUN addgroup -S -g 111 untrusted
# system user, no password, no home dir, no shell, uid 111, group untrusted
RUN adduser -S -D -H -s /bin/false -u 111 -G untrusted untrusted


# copy built pyrod binary in
COPY target/x86_64-unknown-linux-musl/release/pyrod /bin

# install java compiler
# jdk 17 is newest in alpine 3.18
RUN apk update && apk add openjdk17

# copy this image's filesystem to the mounted filesystem when ran
CMD for d in bin etc lib root sbin usr; do tar c "/$d" | tar x -C /rootfs; done && \
    for d in dev proc run sys var; do mkdir /rootfs/${d}; done
