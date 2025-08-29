#!/usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'
set -x

download_cloud_hypervisor() {
      wget "https://github.com/cloud-hypervisor/cloud-hypervisor/releases/download/v51.1/cloud-hypervisor-static"
      chmod +x cloud-hypervisor-static
      wget "https://github.com/cloud-hypervisor/edk2/releases/download/ch-a54f262b09/CLOUDHV.fd"
}

create_disk() {
      rm -rf ch.img
      truncate -s 1G ch.img
      sudo sfdisk ch.img <<EOF
label: gpt
type=uefi
EOF

      lo=$(sudo losetup --show -Pf ch.img)
      p1="${lo}p1"

      sudo mkfs.fat "$p1"
      sudo losetup --detach "$lo"
}

copy_to_disk() {
      lo=$(sudo losetup --show -Pf ch.img)
      p1="${lo}p1"

      sudo mount "$p1" /mnt
      sudo mkdir -p /mnt/efi/boot

      sudo cp target/release/hermit-loader-x86_64.efi /mnt/efi/boot/bootx64.efi
      sudo cp hermit-app /mnt/efi/boot/hermit-app

      sudo umount /mnt
      sudo losetup --detach "$lo"
}

build_app() {
      cd ../hermit-rs
      export HERMIT_LOG_LEVEL_FILTER=trace
      cargo build --target=x86_64-unknown-hermit -Zbuild-std=std,panic_abort --package httpd --features hermit/virtio-net
      cp target/x86_64-unknown-hermit/debug/httpd ../loader/hermit-app
      cd -
}

# download_cloud_hypervisor

# create_disk

build_app

cargo xtask build --target x86_64-uefi --release

copy_to_disk

sudo ./cloud-hypervisor-static \
      --kernel ./CLOUDHV.fd \
      --memory size=4G \
      --serial tty \
      --console off \
      --net "tap=,mac=,ip=10.0.5.1,mask=255.255.255.0" \
      --disk path=ch.img \
| tee cloud-hypervisor.out
