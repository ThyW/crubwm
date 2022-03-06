#!/bin/bash

NDISPLAY=:1

Xephyr -ac -screen 800x600 +extension GLX "${NDISPLAY}" &
xephyr=$!

export DISPLAY="${NDISPLAY}"

sleep 1

cargo run -- --config test-config
wm=$!

if [[ -e /etc/nixos ]] ; then
    "$(nix eval --raw nixpkgs-unstable#bashInteractive)/bin/bash"
else
    bash
fi

kill "${wm}"
kill "${xephyr}"
