#! /bin/sh
# designed to be run *within* the container to extract it's rootfs to a mounted directory
for d in bin etc lib root sbin usr; do tar c "/$d" | tar x -C /rootfs; done
for dir in dev proc run sys var; do mkdir /rootfs/${dir}; done