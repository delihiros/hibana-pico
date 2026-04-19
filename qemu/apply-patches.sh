#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
    echo "usage: $0 /path/to/qemu" >&2
    exit 1
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
patch_dir="$script_dir/patches"
overlay_dir="$script_dir/overlay"
qemu_dir="$1"

while IFS= read -r source; do
    rel="${source#$overlay_dir/}"
    dest="$qemu_dir/$rel"
    mkdir -p "$(dirname "$dest")"
    cp "$source" "$dest"
done < <(find "$overlay_dir" -type f | sort)

while IFS= read -r patch; do
    [ -n "$patch" ] || continue
    git -C "$qemu_dir" apply "$patch_dir/$patch"
done < "$patch_dir/series"
