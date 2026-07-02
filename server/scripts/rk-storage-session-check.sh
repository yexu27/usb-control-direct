#!/usr/bin/env bash
set -u

NBD_POOL_SIZE="${NBD_POOL_SIZE:-4}"
MOUNT_BASE="${MOUNT_BASE:-/mnt/usb_raw}"
GADGET_ROOT="${GADGET_ROOT:-/sys/kernel/config/usb_gadget}"

section() {
  printf '\n== %s ==\n' "$1"
}

read_file_or_empty() {
  local path="$1"
  if [ -r "$path" ]; then
    tr -d '\n' < "$path"
  fi
}

section "NBD module"
if [ -r /sys/module/nbd/parameters/max_part ]; then
  max_part="$(read_file_or_empty /sys/module/nbd/parameters/max_part)"
  printf 'max_part=%s\n' "$max_part"
  if [ "$max_part" != "0" ]; then
    printf 'warning: production image should set nbd.max_part=0 to avoid nbdXpN udev storms\n'
  fi
else
  printf 'max_part=unavailable\n'
fi

section "Mass storage LUN backing"
find "$GADGET_ROOT" -path '*/functions/mass_storage*/lun.*/file' -type f 2>/dev/null | sort | while read -r lun_file; do
  printf '%s -> ' "$lun_file"
  read_file_or_empty "$lun_file"
  printf '\n'
done

section "NBD pool"
idx=0
while [ "$idx" -lt "$NBD_POOL_SIZE" ]; do
  dev="/dev/nbd$idx"
  sys="/sys/block/nbd$idx"
  pid=""
  size=""
  if [ -r "$sys/pid" ]; then
    pid="$(read_file_or_empty "$sys/pid")"
  fi
  if [ -r "$sys/size" ]; then
    size="$(read_file_or_empty "$sys/size")"
  fi
  printf '%s exists=%s pid=%s size=%s\n' "$dev" "$([ -e "$dev" ] && printf yes || printf no)" "${pid:-none}" "${size:-unknown}"
  find /dev -maxdepth 1 -name "nbd${idx}p*" -print 2>/dev/null | sort
  idx=$((idx + 1))
done

section "Raw USB mounts"
if grep " ${MOUNT_BASE}/" /proc/mounts; then
  :
else
  printf 'none under %s\n' "$MOUNT_BASE"
fi

section "Recent NBD partition dmesg"
if command -v dmesg >/dev/null 2>&1; then
  dmesg | tail -n 300 | grep -E 'nbd[0-9]+: p[0-9]+' || printf 'none in recent dmesg\n'
else
  printf 'dmesg unavailable\n'
fi

section "systemd-udevd"
if command -v ps >/dev/null 2>&1; then
  ps -eo pid,comm,pcpu,pmem,args | grep '[s]ystemd-udevd' || printf 'systemd-udevd not found\n'
else
  printf 'ps unavailable\n'
fi
