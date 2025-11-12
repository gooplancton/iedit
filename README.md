# iedit

<img src="demo.gif" alt="Demo" width="1200" height="600">

iedit (short for **i**nline **edit**or) is a compact, terminal-first text editor that doesn't take over your whole terminal. Instead of switching to the alternate screen, iedit opens directly in your terminal's scrollback underneath the current line — using whatever space is available and honoring a configurable minimum height. It's fast, portable, and focused on quick edits and single-file workflows.


## Why iedit?

- Edit files quickly without losing your terminal context or history.
- Inline editing: the editor appears right below the cursor using available scrollback space (and will grow to at least your configured `min_lines`).
- Small, keyboard-driven UX for fast navigation and edits.
- Easy syntax highlighting via nano-style `.nanorc` files (so adding support for new languages is simple).
- Run the current file from inside the editor and view its output in the editor UI.

This project is ideal when you want the convenience of a quick edit without the context switch of a full-screen editor.

## Highlights

- Inline layout: iedit measures the terminal size and cursor position and prints newlines to create a workspace in the scrollback.
- Syntax highlighting: built-in highlighters for Rust, Python and JavaScript + a loader for nano `.nanorc` files. If you have a custom `.nanorc` for a language, drop it in a configured directory and iedit can use it.
- Run inside the editor: press the execute chord to run the file in a background thread; iedit captures stdout and stderr and stores the output in an internal buffer which you can switch to and from with a simple key combination.
- Helpful chords & keybindings: compact, discoverable keybinding popups are available in-editor. The editor can show a help popup listing common bindings.

## Quick start

Run the portable binary (download from Releases) or install via Homebrew:

brew install gooplancton/iedit/iedit

Or run the included debug binary while working from the repo:

cargo run --release -- <path-to-file> [open_line]

If run without a path, iedit opens an empty buffer.

## Basic usage

- Open a file inline below your current terminal cursor:

iedit some_file.rs [open_line]

## Executing the file from inside the editor

iedit can pass the current file as an argument to a background thread and capture its stdout/stderr.

<img src="demo-execute.gif" alt="Demo-Execute" width="1200" height="600">

After execution finishes iedit stores the output and lets you view it from the editor.

## Configuration

Place `~/.iedit.conf` to override defaults. The config structure (see `iedit_editor/src/config.rs`) supports:

- min_lines: minimum editor height in lines (0 keeps adaptive behavior; a positive value reserves that many lines)
- horizontal_margin, vertical_margin: UI margins
- tab_size, tab_emit_spaces: tab rendering and whether tabs insert spaces
- show_line_numbers, show_keybindings: toggles for UI helpers
- confirm_quit_unsaved_changes: prompt before quitting with unsaved changes
- enable_syntax_highlighting: enable/disable highlighting
- syntax_highlighting_dir: optional directory to load custom `*.nanorc` files

## Syntax highlighting

iedit ships builtin highlighting for a few languages (Rust, Python, JavaScript) implemented with a small set of regex-based rules in `iedit_editor/src/editor/highlight.rs`.

To add or reuse existing highlights you can point `syntax_highlighting_dir` to a directory containing `*.nanorc` files. iedit understands the `color` and `icolor` directives from nanorc and maps a small set of colors (and #RRGGBB values) into terminal RGB escapes.

This makes adding new languages as simple as dropping a `.nanorc` named after the file extension (for example `py.nanorc`) in your chosen directory.

## Keybindings & chords

iedit uses simple control-key prefixes and a small chord system for compact commands.

- Ctrl-s: save
- Ctrl-q: quit
- Ctrl-t: display help popup
- Ctrl-k: enter chord mode (then press `x` to execute, `l` for line operations, etc.)
- Ctrl-g: goto line
- Ctrl-f/Ctrl-b: find forward/backward

Chord examples:
- Ctrl-k x x — run using the automatically inferred runner (either shebang line or file extension)
- Ctrl-k x p — run file with python3
- Ctrl-k l n — toggle line numbers

Hints are automatically displayed as the chords are being entered.

## Installation

- Homebrew:

	brew install gooplancton/iedit/iedit

- Releases: portable static builds are provided in the GitHub Releases page (pick the appropriate platform and drop the binary on your PATH).

- From source:

	cargo build --release

Then copy `target/release/iedit` to somewhere on your PATH.

## Contributing

Contributions, bug reports and small PRs welcome. If you're adding syntax files, please prefer `.nanorc` format and include a short test file.

Issues and PRs: https://github.com/gooplancton/iedit

## License

See the repository `LICENSE` file.
