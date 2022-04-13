#!/bin/bash

# make sure that every other Xephyr and window manager instance is killed before running
killall Xephyr
killall crubwm

# exit on first error
set -e

# the display we want to use
NDISPLAY=:1

# run Xephyr
Xephyr -ac -screen 800x600 +extension GLX "${NDISPLAY}" &
# save it's pid
xephyr=$!

# run the wm on anoter display
export DISPLAY="${NDISPLAY}"

# run the command specified in the command line arguments given to this script
wm=""
if [[ $1 = "--default" ]]; then
    cargo run -- --config test-config
    wm=$!
else
    $@
    wm=$!
fi

# run a new bash instance
bash
xsetroot -solid "#ffffff"

# kill everything at the end
kill -9 "${wm}"
kill -9 "${xephyr}"
