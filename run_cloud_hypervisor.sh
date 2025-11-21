#!/usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'
set -x

create_disk() {
      rm -rf esp.fat
      truncate -s 1G esp.fat
      sudo fdisk esp.fat <<EOF
g
n



t
1
w
EOF

      lo=$(sudo losetup --show -Pf esp.fat)
      p1="${lo}p1"

      sudo mkfs.fat "$p1"
      sudo losetup --detach "$lo"
}

copy_to_disk() {
      lo=$(sudo losetup --show -Pf esp.fat)
      p1="${lo}p1"

      sudo mount "$p1" /mnt
      sudo mkdir -p /mnt/efi/boot

      sudo cp target/release/hermit-loader-x86_64.efi /mnt/efi/boot/bootx64.efi
      sudo cp hermit-app /mnt/efi/boot/hermit-app

      sudo umount /mnt
      sudo losetup --detach "$lo"
}

# create_disk

(
      cd ../hermit-rs
      export HERMIT_LOG_LEVEL_FILTER=trace
      cargo build --target=x86_64-unknown-hermit -Zbuild-std=std,panic_abort --package httpd --features hermit/virtio-net
      cp target/x86_64-unknown-hermit/debug/httpd ../loader/hermit-app
)

cargo xtask build --target x86_64-uefi --release

copy_to_disk

sudo ./cloud-hypervisor-static \
      --kernel ./CLOUDHV.fd \
      --memory size=4G \
      --serial tty \
      --console off \
      --net "tap=,mac=,ip=10.0.5.1,mask=255.255.255.0" \
      --disk path=esp.fat | tee foo
