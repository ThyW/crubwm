# Table of contents
This document discusses how to configure the `crubwm` window manager. It is split into multiple sections:
- [Configuration](#Configuration)
- [Keybindings](#Keybinds)
- [Hooks](#Hooks)
- [WM settings](#WMsettings)
- [Workspace settings](#Workspace_settings)
- [Bar settings](#Bar_settings)

## Configuration
The configuration of `crubwm` is loaded from a configuration file on startup. The configuration file is parsed, verified and then used. After that the rest of the window manager's utilities are run and setup. By default, `crubwm` looks for the configuration file located on the following path: `$XDG_CONFIG/crubwm/config`. If the file is not found, `crubwm` will attempt to create it and save the default settings into it. Alternatively, a `--config` command line option followed by the `path` to a desired configuration file can be used when running crubwm. The format of the configuration file shall be discussed in the following sections.

### Configuration format
Each line is a single configuration option. A line starts with a keyword specifying what type of option it is. The general format of a configuration line is as follows:
```
[keyword] [list of arguments and suboptions...]
```

The configuration currently recognizes the following keywords:
- `keybind`
- `hook`
- `set`
- `workspace_set`
- `bar_set`

The options specified by each keywords are discussed in the following sections.

## Keybinds
The format for writing keybinds is as follows:
```
keybind [list of keys] [action] [action arugments, ...]
```

The line starts with the `keybind` keyword. It is followed by a string representing the keys which are being bound, an example might be: `<Alt><Shift>q` or `<LCtrl><cr>`. Notice, that some keys are passed in inside of `<>` angled brackets. These are the special keys. Following is a table of the special keys.

| Key                  | Config representation                     |
|-------------------   | -----------------------------------------   |
| Super Key            | `<mod>`, `<super_l>`                        |
| Left Control Key     | `<ctrl>`, `<control_l>`, `<lctrl>`          |
| Right Control Key    | `<control_r>`, `<rctrl>`                    |
| Escape Key           | `<esc>`, `<escape>`			     |
| Function Keys        | `<f1>`, `<f2>`, ... `<f11>`, `<f12>`, `<fn>`|
| PrintScreen Key      | `<print>`                                   |
| Scroll Lock Key      | `<scroll_lock>`                             |
| Pause Key            | `<pause>`                                   |
| Backtick             | \`, `<backtick>`, `<grave>`                 |
| Minus                | \-, `<minus>`                               |
| Equals               | =, `<equal>`                                |
| Backspace            | `<backspace>`                               |
| Insert               | `<insert>`                                  |
| Home                 | `<home>`                                    |
| Page Up              | `<pgup>`, `<pageup>`, `<prior>`             |
| Numlock              | `<numlock>`                                 |
| Number Pad Divide    | `<numdivide>`                               |
| Number Pad Multiply  | `<numdivide>`                               |
| Number Pad Subtract  | `<numsubtract>`                             |
| Number Pad Numbers   | `<num1>`, `<num2>`, ...                     |
| Number Pad Enter     | `<numenter>`                                |
| Number Pad Decimal   | `<numdecimal>`                              |
| Tab Key              | `<tab>`                                     |
| Delete               | `<delete>`                                  |
| End                  | `<end>`                                     |
| Page Down            | `<pgdown>`, `<pagedown>`, `<next>`          |
| Caps lock key        | `<caps_lock>`, `<caps>`                     |
| Enter key            | `<cr>`, `<enter>`, `<return>`               |
| Left Shift key       | `<shift>`, `<shift_l>`, `<lshift>`          |
| Right Shift key      | `<shift_r>`, `<rshift>`                     |
| Arrow keys           | `<up>`, `<down>`, `<left>`, `<right>`       |
| Space key            | `<space>`                                   |
| Menu key             | `<menu>`                                    |
| Left Alt key         | `<alt_l>`, `<alt>`                          |
| Left Alt key         | `<alt_r>`, `<ralt>`                         |

It is also useful to know that the keys are not case sensitive, so `<ctrl>` is the same as `<Ctrl>` as well as `<CTRL>`.

After the keys comes the action argument. Actions are triggered by keybinds and they do some stuff based on their type. Some action also have arguments. Following is a list of the currently supported actions with their descriptions and optional arguments.

- `noop` - don't do anything
- `kill` - this kills the currently focused client.
- `execute [...]` - execute a command on the host system.
    - this action takes a list of arguments which are then passed to `/bin/bash -c` and executed.
- `goto [workspace_id]` - switch to a specified workspace.
    - this action takes a workspace identifier(number) as an argument.
- `move [workspace_id]` - move the currently focused client to the specified workspace.
    - this action takes a workspace identifier(number) as an argument.
- `focus [direction]` - focus the next or previous client in the current workspace based on direction.
    - this action takes a direction, a string of either `"next"` or `"previous"` as an argument.
- `change_layout [layout]` - attempt to switch to the layout specified.
    - this action takes a layout name as an argument, more on layouts in their [section](#WM settings)
- `cycle_layout` - move to the next layout.
- `toggle_float` - make the currently focused client float or put it back into tiled mode.
- `swap [direction]` - swap a client with its next or previous neighbour.
    - this action takes a direction, a string of either `"next"` or `"previous"` as an argument.
- `reload_config` - reload the currently loaded configuration file.

Following is a list of all of the default keybinds.
```
keybind "<Mod><Enter>" execute "xterm"
keybind "<Mod>k" kill
keybind "<Mod>1" goto 1
keybind "<Mod>2" goto 2
keybind "<Mod>3" goto 3
keybind "<Mod>4" goto 4
keybind "<Mod>5" goto 5
keybind "<Mod>6" goto 6
keybind "<Mod>7" goto 7
keybind "<Mod>8" goto 8
keybind "<Mod>9" goto 9
keybind "<Mod>0" goto 10
keybind "<Mod><Shift>1" move 1
keybind "<Mod><Shift>2" move 2
keybind "<Mod><Shift>3" move 3
keybind "<Mod><Shift>4" move 4
keybind "<Mod><Shift>5" move 5
keybind "<Mod><Shift>6" move 6
keybind "<Mod><Shift>7" move 7
keybind "<Mod><Shift>8" move 8
keybind "<Mod><Shift>9" move 9
keybind "<Mod><Shift>0" move 10
keybind "<Mod>l" focus next
keybind "<Mod>h" focus previous
keybind "<Mod><Shift>l" swap next
keybind "<Mod><Shift>h" swap previous
keybind "<Mod>s" cycle_layout
keybind "<Mod><space>" toggle_float
keybind "<Mod><Shift>r" reload_config
```

## Hooks
Hooks specify commands which are triggered when some events happen. In our case, these events are either initial startup of the window manager or a reload of the configuration file. These two types are specified by the `startup` and `always` arguments to the `hook` keyword. Another type of argument which should be specified is the synchronicity of the command executed. This sub argument also has two possible values: `sync` and `async`. The difference is quite obvious, `sync` halts the window manager's operation until the command exists and `async` just runs the command and doesn't care about when or how it exits. The general format for writing hooks is as follows:

```
hook [always | startup] [sync | async] [command/s to execute using /bin/sh -c ...]
```

An example comes from the default configuration files, where `xsetroot` is used to set the root window background color.
```
hook startup sync "xsetroot -solid '#282828'"
```

It is also important to note that `""` are not necessary, ever. They can, however, be useful when striving for higher readability of the configuration file. 

## WM settings
Window manager settings are settings general to the whole of window manager. Their format is very simple, it's just the `set` keyword followed by the setting name and its value.
```
set [setting name] [setting value]
```

Following is a list of all currently supported options and their values.
- `border` - a window have a border?
    - possible values are `true` and `false`
- `border_size` - how thick should the window border be?
    - takes an **unsigned integer**: `1`, `20`
- `border_color` - what color should the border be?
    - a hexadecimal RGB value starting with `#`: `#282828`
- `display_name` - name of the X11 display this WM should run on
    - a string, if the default display should be used, pass in an empty string
- `gap_top` , `gap_bottom`, `gap_left`,  `gap_right` - should there be gaps between windows for?
    - possible values are `true` and `false`
- `gap_top_size` , `gap_bottom_size`, `gap_left_size`,  `gap_right_size` - how big should the gaps between the windows should be?
    - takes an **unsigned integer**: `1`, `10`
    - if 0, there are no gaps

## Workspace settings
Workspace settings are settings which are native to a single, specified workspace. By default, crubwm comes with 10 predefined workspaces, their ids ranging are 1 through 10. When adding a workspace setting, a workspace id must be specified.
```
workspace_set [setting name] [setting value/s]
```

Here is a list of all the currently supported workspace settings.

- `name` - custom identifier for the workspace, this can be used to when drawing the status bar.
    - a string value
- `allowed_layouts` - a list of layouts that are available on the workspace.
    - possible values are: `all`, `tiling_equal_horizontal`, `tiling_equal_vertical`, `tiling_master_stack`
- `monitor` - workspaces are not dynamic, they have to be defined on a given monitor before running the window manager. This setting takes the monitor number and attempts to place the workspace on that monitor.
- `default_container_type` - ***[Currently not supported]*** signals what type a window should be when created.
    - possible values: `float`, `in_layout`

## Bar settings
