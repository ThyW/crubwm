# About
CrubWM is a hobby project, who's sole purpose is for me to learn how the X protocol works and use it to implement a window manager, in Rust of course. It is still by all means, a work in progress and a mess. See the [TOOD](#todo) section to see what is done and what awaits further attention.

# Installation
If you are building from source, make sure you have `cargo` installed. Also, make sure you have `Xlib` and `xcb` installed as well.

After cloning this repository using `git`:

```
git clone https://github.com/ThyW/crubwm.git
```

`cd` into the newly created directory: `cd crubwm/` and run:
```
cargo build --release
```

This will compile the window manager and output the resulting compiled binary file into the `target/release/` directory. You can than symlink it to a place of your liking, for example `~/.local/bin`(or somewhere else in your PATH) for that matter.

After that, you can just use the display manager of your choice to run the window manager. (I only ever tested this using `xinit`)

# Configuration
Configuration is done using the `~/config/crubwm/config` configuration file. This file automatically generates for you, if not found on the first run of this program.

Alternatively, you can specify the `--config <file>`, where `<file>` is a path to a configuration file.

## Defaults

```
keybind <mod><enter> exec xterm
keybind <mod><shift>q kill

keybind <mod>1 goto 1
keybind <mod>2 goto 2
keybind <mod>3 goto 3
keybind <mod>4 goto 4
keybind <mod>5 goto 5
keybind <mod>6 goto 6
keybind <mod>7 goto 7
keybind <mod>8 goto 8
keybind <mod>9 goto 9

keybind <mod><shift>1 move 1
keybind <mod><shift>2 move 2
keybind <mod><shift>3 move 3
keybind <mod><shift>4 move 4
keybind <mod><shift>5 move 5
keybind <mod><shift>6 move 6
keybind <mod><shift>7 move 7
keybind <mod><shift>8 move 8
keybind <mod><shift>9 move 9

keybind <mod>l focus next
keybind <mod>h focus previous

keybind <mod><shift>l swap next
keybind <mod><shift>h swap previous

keybind <mod>s cycle_layout
keybind <mod><space> toggle_float
```

This are the default binds which will be generated for you.

## More configuration options
TODO

# TODO
- [x] tiling window management with extensible layouts
- [x] basic keybinds and action support(switching focused clients, moving clients between workspaces, switching workspaces etc...)
- [x] multiple monitor support - more or less works, can now use it as a daily driver
- [x] floating windows
- [x] graphics, text and window decorations
- [x] stacking layout
- [ ] bar
- [ ] EWMH and ICCC compliance
- [x] startup hooks
- [x] config reloading on the fly
- [ ] docs - most of the project has doc strings
