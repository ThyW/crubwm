# Table of contents
This document discusses how to configure the `crubwm` window manager. It is split into multiple sections:
- [Configuration](#Configuration)
- [Keybindings](#Keybinds)
- [Hooks](#Hooks)
- [WM settings](#wm-settings)
- [Workspace settings](#workspace-settings)
- [Bar settings](#bar-settings)
    - [Widgets](#widget-segment)
    - [Window title](#title-segment)
    - [Workspace info](#workspace-info-segment)

## Configuration
The configuration of `crubwm` is loaded from a configuration file on startup. The configuration file is parsed, verified and then used. After that, the rest of the window manager's utilities are run and setup. By default, `crubwm` looks for the configuration file located on the following path: `$XDG_CONFIG/crubwm/config`. If the file is not found, `crubwm` will attempt to create it and save the default settings into it. Alternatively, a `--config` command line argument followed by the `path` to a desired configuration file can be used when running crubwm. The format of the configuration file will be discussed in the following sections.

### Configuration format
Each line is a single single directive to the parser, currently there is no way to split a line. A line starts with a keyword specifying what type of option it is. The general format of a configuration line is as follows:
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

The `""` characters can be used to enclose a multi word string which tells the parser to parse it as a single value. Sometimes they are not necessary, but they can be used everywhere for better readability of the configuration file.

## Keybinds
The format for writing keybinds is as follows:
```
keybind [list of keys] [action] [action arugments, ...]
```

The line starts with the `keybind` keyword. It is followed by a string representing the keys which are being bound, an example might be: `<Alt><Shift>q` or `<LCtrl><cr>`. Notice, that some keys are passed in inside of `<>` angled brackets. These are the special keys. Following is a table of the special keys.

| Key                  | Config representation                       |
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
| Right Alt key         | `<alt_r>`, `<ralt>`                        |

It is also useful to know that the keys are not case sensitive, so `<ctrl>` is the same as `<Ctrl>` as well as `<CTRL>`.

After the keys comes the action argument. Actions are triggered by keybinds and they do some stuff based on their type. Some action also have arguments. Following is a list of the currently supported actions with their descriptions and optional arguments.

- `noop` - don't do anything
- `kill` - this kills the currently focused client.
- `execute [...]` - execute a command on the host system.
    - this action takes a list of arguments which are then passed to `/bin/sh -c` and executed.
- `goto [workspace_id]` - switch to a specified workspace.
    - this action takes a workspace identifier(number) as an argument.
- `move [workspace_id]` - move the currently focused client to the specified workspace.
    - this action takes a workspace identifier(number) as an argument.
- `focus [direction]` - focus the next or previous client in the current workspace based on direction.
    - this action takes a direction, a string, either `"next"` or `"previous"` as an argument.
- `change_layout [layout]` - attempt to switch to the layout specified.
    - this action takes a layout name as an argument, more on layouts in their [section](#wm-settings)
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

## WM settings
Window manager settings are settings general to the whole window manager. Their format is very simple, it's just the `set` keyword followed by the setting name and its value.
```
set [setting name] [setting value]
```

Following is a list of all currently supported options and their values.
- `border` - indicates, whether windows should be drawn with borders.
    - possible values are `true` and `false`
- `border_size` - indicates the thickness of the border.
    - takes an **unsigned integer**: `1`, `20`
- `border_color` - indicates the color of the border.
    - a hexadecimal RGB value starting with `#`: `#282828`
- `display_name` - name of the X11 display this WM should run on
    - a string, if the default display should be used, pass in an empty string
- `gap_top` , `gap_bottom`, `gap_left`,  `gap_right` - should there be gaps between windows for?
    - possible values are `true` and `false`
- `gap_top_size` , `gap_bottom_size`, `gap_left_size`,  `gap_right_size` - how big should the gaps between the windows should be?
    - takes an **unsigned integer**: `1`, `10`
    - value `0` does not display gaps.
- `log_level` - how much information should be logged.
    - possible are three values: `0` - disable logging; `1` - light logging; `2` - log everything.
- `log_file` - path to a log file.
    - this takes a string, however, there are 2 reserved strings: `STDOUT` and `STDERR` which instead of writing to a file, write to `stdin` and `stdout` respectively.

## Workspace settings
Workspace settings are settings which are native to a single, specified workspace. By default, crubwm comes with 10 predefined workspaces, their ids ranging from 1 through to 10. When adding a workspace setting, a workspace id must be specified.
```
workspace_set [worksapce id] [setting name] [setting value/s]
```

Here is a list of all the currently supported workspace settings.

- `name` - custom identifier for the workspace, this can be used when drawing the status bar.
    - a string value
- `allowed_layouts` - a list of layouts that are available on the workspace.
    - possible values are: `all`, `tiling_equal_horizontal`, `tiling_equal_vertical`, `tiling_master_stack`, `stacking`
- `monitor` - workspaces are not dynamic, they have to be defined on a given monitor before running the window manager. This setting takes the monitor number and attempts to place the workspace on that monitor.
- `default_container_type` - signals what type a window should be when created.
    - possible values: `float`, `in_layout`

## Bar settings
This section discusses the configuration process of the inbuilt status bar. A bar has a unique identifier and is always associated with a monitor. A bar has a number of segments which provide the information which the user desires. By default, there are no segments and the user has to define them according to their liking. More on the different segment types and settings in further sections.

First, let's start with the general bar settings. Format is as follows:
```
bar_set [bar identifier] [setting/option] [value/s...]
```

Currently supported bar settings are:
- `monitor` - monitor identifier of the monitor this bar should be placed on.
    - takes an unsigned integer as an argument.
- `font_size` - the size of all the fonts in the bar.
    - takes an unsigned integer as an argument.
- `height` - maximum height of the bar. This field does not need to be set, the height is inferred from the font size.
    - takes an unsigned integer as an argument.
- `background_color` - color of the bar background.
    - takes a 7 character string, a hex color beginning with `#`.
- `location_top` - should the bar be placed on top or bottom of the screen.
    - takes either `true` or `false` as arguments.

A special case is the bar identifier. By default, there are no bars. A bar with an ID is only created when a `bar_identifier` is used for the first time.

### Segments
Segments are added to the bar using this syntax:
```
bar_set [bar identifier] segment add [segment type] [segment name] [segment position]
```

Currently supported segment types are:
- [`widget`](#widget-segment) 
- [`workspace`](#workspace-info-segment) 
- [`title`](#title-segment)

Segment name is a unique identifier of that segment, it will be used when further configuring that segment. Segment position tells the bar where the segment should be rendered. Currently supported segment positions are:
- `left`
- `middle`
- `right`

### Widget segment
A widget segment holds widgets, user defined structures which display user defined information and are periodically updated. An example of a widget might be a widget which shows the current time(minute and hour). To not waste system resources it should only be updated every 60 seconds or so. On the other hand, there might be a CPU utilization widget which should be updated every second for the most accurate and up to date information.

A new widget segment is added using:
```
bar_set [bar identifier] segment add "widget" [segment identifier] [segment position]
```

This creates an empty widget segment which is not really useful. To make the segment useful, we need to populate it with widgets. This is done like so:
```
bar_set [bar identifier] widget add [widget segment identifier] [widget identifier] [setting and value pairs...]
```

Currently supported settings are:
- `command` - the command to be executed every `update_time` number of seconds. This command's output is then used as the value of this widget.
    - a string, the command and its arguments which are then passed to `/bin/sh -c `
- `icon` - a string of characters which will be displayed when rendering the widget.
- `icon_color` - color of the `icon` text.
    - takes a 7 character string, a hex color beginning with `#`.
- `value_color` - color of the text displaying the value of the widget.
    - takes a 7 character string, a hex color beginning with `#`.
- `background_color` - the background color of the entire widget.
    - takes a 7 character string, a hex color beginning with `#`.
- `separator` - a string which will be used to separate this widget from other widgets.
- `separator_color` - the color of the separator text.
    - takes a 7 character string, a hex color beginning with `#`.
- `update_time` - how often should this widget be updated.
    - takes an unsigned integer.
- `font` - what font should be used for drawing this widget's text.
    - should be in the following format: `fontname:weight=[font weight: either bold or normal]:slant=[font slat: either italic or normal]`
- `format` - a string which describes the overall format of how the widget will be rendered.
    - there are three known values in the format string: `{icon}`, `{value}`, `{separator}`
    - the default format looks like this: `{separator} {icon} {value} {separator}`

### Workspace info segment
The workspace info segment shows the user information about the current state of the window manager workspaces. It can show workspace name, identifier, which workspace is currently focused and which workspace requires the users attention.

It is added into the bar like so:
```
bar_set [bar identifier] segment add "workspace" [workspace segment identifier] [segment position]
```

After that, the following settings can be set on the segment using:
```
bar_set [bar identifier] workspace set [workspace segment identifier] [setting-value pairs...]
```

- `focused_foreground_color` - text color of the currently focused workspace
    - takes a 7 character string, a hex color beginning with `#`.
- `focused_background_color` - background color of the currently focused workspace
    - takes a 7 character string, a hex color beginning with `#`.
- `normal_foreground_color` - text color of the currently unfocused workspace
    - takes a 7 character string, a hex color beginning with `#`.
- `normal_background_color` - background color of the currently unfocused workspace
    - takes a 7 character string, a hex color beginning with `#`.
- `font` - font of the text drawn.
    - should be in the following format: `fontname:weight=[font weight: either bold or normal]:slant=[font slat: either italic or normal]`
- `format` - a string which describes the overall format of how the text of each workspace will be rendered.
    - default: ` {name}:{id} `, where `{name}` is the user defined name of the workspace and `{id}` is the identifier of the workspace

### Title segment
This segment shows the window title of the currently focused window. It is added the same way previous segments were added:
```
bar_set [bar identifier] segment add "title" [segment identifier] [position]
```

A title bar also has a few settings which are set using:
```
bar_set [bar identifier] title set [title identifier] [setting-value pairs...]
```

- `font` - font of the text drawn.
    - should be in the following format: `fontname:weight=[font weight: either bold or normal]:slant=[font slat: either italic or normal]`
- `foreground_color` - text color.
    - takes a 7 character string, a hex color beginning with `#`.
- `background_color` - background color.
    - takes a 7 character string, a hex color beginning with `#`.
