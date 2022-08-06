#!/bin/bash

# make sure that every other Xephyr and window manager instance is killed before running
killall Xephyr

# exit on first error
set -e

# the display we want to use
NDISPLAY=:1

# run Xephyr
Xephyr -ac -screen 1280x720 +extension GLX "${NDISPLAY}" 2>/dev/null &
# save it's pid
xephyr=$!

# run the wm on anoter display
DISPLAY="${NDISPLAY}"

# run the command specified in the command line arguments given to this script
wm=""
if [[ $1 = "debug" ]]; then
    if [[ $2 = "-c" ]]; then
	cargo run -- --config $3
	wm=$!
    else 
	cargo run -- --config test-config
	wm=$!
    fi
elif [[ $1 = "release" ]]; then
    cargo run --release -- --config test-config
    wm=$!
else
    $@
    wm=$!
fi

echo $wm

# run a new bash instance
bash
xsetroot -solid '#222222'

# kill everything at the end
kill -9 "${wm}"
kill -9 "${xephyr}"
