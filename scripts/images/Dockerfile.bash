FROM bash:5.2

# create service user for untrusted processes to run under
# system group, gid 111
RUN addgroup -S -g 111 untrusted
# system user, no password, no home dir, no shell, uid 111, group untrusted
RUN adduser -S -D -H -s /bin/false -u 111 -G untrusted untrusted

COPY target/x86_64-unknown-linux-musl/release/pyrod /bin

CMD for d in bin etc lib root sbin usr; do tar c "/$d" | tar x -C /rootfs; done && \
    for d in dev proc run sys var; do mkdir /rootfs/${d}; done