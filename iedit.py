#!/usr/bin/env python3

import sys
import os
import termios
import tty
import select
import re

NOOB_MODE = 0
NORMAL_MODE = 1
INSERT_MODE = 2
VISUAL_MODE = 3
COMMAND_MODE = 4

mode_status_text = {
    NOOB_MODE: "",
    NORMAL_MODE: "[NOR] ",
    INSERT_MODE: "[INS] ",
    VISUAL_MODE: "[VIS] ",
    COMMAND_MODE: ""
}

class InlineFileEditor:
    def __init__(self, filename, lines_to_show=10, vim_mode=False):
        self.filename = filename
        self.lines_to_show = lines_to_show
        self.current_line = 0
        self.file_lines = []
        self.status_lines = 2  # Separator + status text
        self.horizontal_margin = 2
        self.vertical_margin = 5
        self.status_text = ""

        # Cursor position within the current line
        self.cursor_pos = 0
        self.modified = False
        self.is_moving_forward = True
        self.cursor_rel_x = 0
        self.cursor_rel_y = 0

        # Track previous state for minimal redraws
        self.prev_line = 0
        self.prev_cursor_pos = 0
        self.prev_start_line = 0

        self.mode = NORMAL_MODE if vim_mode else NOOB_MODE
        self.command_text = ""
        self.vim_motion_counter = 0
        self.selection_anchor = None

        # ANSI escape sequences
        self.CURSOR_UP = '\033[{}A'
        self.CURSOR_DOWN = '\033[{}B'
        self.CURSOR_RIGHT = '\033[{}C'
        self.CURSOR_LEFT = '\033[{}D'
        self.CLEAR_LINE = '\033[2K'
        self.CLEAR_BELOW_CURSOR = '\033[J'
        self.CURSOR_TO_COL1 = '\r'
        self.CURSOR_TO_LINE = '\033[{};1H'
        self.SAVE_CURSOR = '\033[s'
        self.RESTORE_CURSOR = '\033[u'
        self.HIDE_CURSOR = '\033[?25l'
        self.SHOW_CURSOR = '\033[?25h'
        self.START_HIGHLIGHT = '\033[47m\033[0;30m'
        self.RESET_HIGHLIGHT = '\033[0m\033[0;37m'

        self.H = '─'

    def load_file(self):
        """Load file contents into memory"""
        try:
            with open(self.filename, 'r', encoding='utf-8', errors='replace') as f:
                self.file_lines = f.readlines()
            # Remove trailing newlines but preserve empty lines
            self.file_lines = [line.rstrip('\n\r') for line in self.file_lines]
            if not self.file_lines:
                self.file_lines = [""]  # Start with empty line if file is empty
            return True
        except Exception as e:
            print(f"Error reading file: {e}")
            return False

    def parse_vim_input(self, char):
        if self.mode == INSERT_MODE:
            return char
        if self.mode == COMMAND_MODE:
            if char == '\x7f':
                self.command_text = self.command_text[0:len(self.command_text)-1]
            else:
                self.command_text += char
            return "PASS"

        if char.isnumeric() and not (char == '0' and self.vim_motion_counter == 0):
            self.vim_motion_counter = self.vim_motion_counter * 10 + int(char)
            return "PASS"
        if char == 'j':
            for _ in range(0, max(self.vim_motion_counter, 1)):
                self.move_cursor_down()

            self.vim_motion_counter = 0
            return "PASS"
        elif char == 'h':
            for _ in range(0, max(self.vim_motion_counter, 1)):
                self.move_cursor_left()

            self.vim_motion_counter = 0
            return "PASS"
        elif char == 'k':
            for _ in range(0, max(self.vim_motion_counter, 1)):
                self.move_cursor_up()

            self.vim_motion_counter = 0
            return "PASS"
        elif char == 'l':
            for _ in range(0, max(self.vim_motion_counter, 1)):
                self.move_cursor_right()

            self.vim_motion_counter = 0
            return "PASS"
        elif char == 'v':
            self.mode = VISUAL_MODE
            self.selection_anchor = (self.current_line, self.cursor_pos)

            return "PASS"
        elif char == 'i':
            self.mode = INSERT_MODE
            return "PASS"
        elif char == "a":
            self.move_cursor_right()
            self.mode = INSERT_MODE
            return "PASS"
        elif char == "w":
            for _ in range(0, max(self.vim_motion_counter, 1)):
                self.move_cursor_forward_regex(r"\s+\S(.?)")

            self.vim_motion_counter = 0
            return "PASS"
        elif char == "b":
            for _ in range(0, max(self.vim_motion_counter, 1)):
                self.move_cursor_backward_regex(r"[\n\s](\S)")

            self.vim_motion_counter = 0
            return "PASS"
        elif char == "0":
            self.cursor_pos = 0
            return "PASS"
        elif char == "$":
            self.move_cursor_forward_regex("($)")
            return "PASS"
        elif char == "e":
            for _ in range(0, max(self.vim_motion_counter, 1)):
                self.move_cursor_forward_regex(r"\S([$\s])")

            self.vim_motion_counter = 0
            return "PASS"
        elif char == ':':
            self.mode = COMMAND_MODE
            return "PASS"

    def get_char(self):
        """Get a single character from stdin without requiring Enter"""
        fd = sys.stdin.fileno()
        old_settings = termios.tcgetattr(fd)
        old_flags = termios.tcgetattr(fd)
        old_flags[0] &= ~termios.IXON  # Disable flow control
        termios.tcsetattr(fd, termios.TCSANOW, old_flags)
        try:
            tty.setcbreak(fd)
            if select.select([sys.stdin], [], [], 0.1)[0]:
                ch = sys.stdin.read(1)
                if ch == '\033':  # Escape sequence
                    if self.mode != NOOB_MODE:
                        self.selection_anchor = None
                        self.vim_motion_counter = 0
                        self.mode = NORMAL_MODE
                        return "PASS"

                    next_chars = sys.stdin.read(2)
                    if next_chars == '[A':
                        return 'UP'
                    elif next_chars == '[B':
                        return 'DOWN_1'
                    elif next_chars == '[C':
                        return 'RIGHT'
                    elif next_chars == '[D':
                        return 'LEFT'
                elif ch == '\x11' and self.mode == NOOB_MODE:  # Ctrl+Q
                    return 'QUIT'
                elif ch == '\x13' and self.mode == NOOB_MODE:  # Ctrl+S
                    return 'SAVE'
                elif ch == '\x7f' and (self.mode == NOOB_MODE or self.mode == INSERT_MODE):  # Backspace
                    return 'BACKSPACE'
                elif ch == '\r' or ch == '\n':  # Enter
                    return 'ENTER'
                else:
                    if self.mode != NOOB_MODE:
                        return self.parse_vim_input(ch)
                    return ch
            return None
        finally:
            termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)

    def move_cursor_up(self):
        """Move cursor up one line"""
        if self.current_line > 0:
            self.is_moving_forward = False
            self.current_line -= 1
            self.cursor_rel_y -= 1
            # Keep cursor position within bounds of new line
            self.cursor_pos = min(self.cursor_pos, len(self.file_lines[self.current_line]))

    def move_cursor_down(self):
        """Move cursor down one line"""
        if self.current_line < len(self.file_lines) - 1:
            self.is_moving_forward = True
            self.current_line += 1
            self.cursor_rel_y += 1
            # Keep cursor position within bounds of new line
            self.cursor_pos = min(self.cursor_pos, len(self.file_lines[self.current_line]))

    def move_cursor_left(self):
        """Move cursor left one character"""
        if self.cursor_pos > 0:
            self.is_moving_forward = False
            self.cursor_pos -= 1

    def move_cursor_right(self):
        """Move cursor right one character"""
        current_line_content = self.file_lines[self.current_line]
        if self.cursor_pos < len(current_line_content):
            self.is_moving_forward = True
            self.cursor_pos += 1

    def move_cursor_forward_regex(self, regex):
        self.is_moving_forward = True
        current_line_content = self.file_lines[self.current_line]
        match = re.search(regex, current_line_content[(self.cursor_pos + 1):])
        if match == None:
            self.cursor_pos = 0
            return self.move_cursor_down()

        group_idx = len(match.groups())
        offset = match.span(group_idx)[0]
        if self.cursor_pos < len(current_line_content):
            self.cursor_pos += offset

    def move_cursor_backward_regex(self, regex):
        self.is_moving_forward = False
        current_line_content = self.file_lines[self.current_line]
        matches = re.finditer(regex, current_line_content[:self.cursor_pos])
        match = get_last_match(matches)
        if match == None:
            self.cursor_pos = 0
            return self.move_cursor_up()

        group_idx = len(match.groups())
        idx = match.span(group_idx)[0]
        if self.cursor_pos < len(current_line_content):
            self.cursor_pos = idx

    def insert_char(self, ch):
        """Insert a character at cursor position"""
        current_line = self.file_lines[self.current_line]
        new_line = current_line[:self.cursor_pos] + ch + current_line[self.cursor_pos:]
        self.file_lines[self.current_line] = new_line
        self.cursor_pos += 1
        self.modified = True
        self.is_moving_forward = True

    def delete_char(self):
        """Delete character before cursor"""
        self.is_moving_forward = False
        if self.cursor_pos > 0:
            current_line = self.file_lines[self.current_line]
            new_line = current_line[:self.cursor_pos-1] + current_line[self.cursor_pos:]
            self.file_lines[self.current_line] = new_line
            self.cursor_pos -= 1
            self.modified = True
        elif self.current_line > 0:
            # At beginning of line - merge with previous line
            prev_line_len = len(self.file_lines[self.current_line - 1])
            self.file_lines[self.current_line - 1] += self.file_lines[self.current_line]
            self.file_lines.pop(self.current_line)
            self.current_line -= 1
            self.cursor_pos = prev_line_len
            self.modified = True

    def insert_newline(self):
        """Insert newline at cursor position (split line)"""
        current_line = self.file_lines[self.current_line]
        before_cursor = current_line[:self.cursor_pos]
        after_cursor = current_line[self.cursor_pos:]

        self.file_lines[self.current_line] = before_cursor
        self.file_lines.insert(self.current_line + 1, after_cursor)

        self.current_line += 1
        self.cursor_pos = 0
        self.modified = True
        self.is_moving_forward = True

    def display_line(self, line_idx, current_line, cursor_pos, selection_range):
        """Display a single line with proper formatting"""
        cols = os.get_terminal_size().columns
        inner_content_width = max(0, cols - 2) + 2
        line_num = line_idx + 1
        prefix = f"{line_num:4d}: "
        term_width = inner_content_width - len(prefix)

        line_text = self.file_lines[line_idx]
        offset = max(self.cursor_pos - term_width + 1, 0)
        content = line_text[offset:offset + term_width]

        if selection_range:
            range_start, range_end = selection_range

            range_start_line_idx, range_start_cursor_pos = range_start
            includes_range_start = line_idx == range_start_line_idx
            if includes_range_start:
                content = strinsert(content, range_start_cursor_pos, self.START_HIGHLIGHT)

            range_end_line_idx, range_end_cursor_pos = range_end
            includes_range_end = line_idx == range_end_line_idx
            if includes_range_end:
                content = strinsert(content, range_end_cursor_pos, self.RESET_HIGHLIGHT)

        # Highlight current line and show cursor
        if line_idx == current_line:
            # Show cursor position
            display_content = content

            # Insert cursor visualization
            cursor_display_pos = min(cursor_pos - offset, len(display_content))
            before_cursor = display_content[:cursor_display_pos]
            at_cursor = display_content[cursor_display_pos] if cursor_display_pos < len(display_content) else " "
            after_cursor = display_content[cursor_display_pos+1:] if cursor_display_pos + 1 < len(display_content) else ""

            max_before = term_width - 1 - len(after_cursor) if after_cursor else term_width - 1
            if len(before_cursor) > max_before:
                before_cursor = before_cursor[len(before_cursor)-max_before:]

            display = before_cursor + "\x1b[7m" + at_cursor + "\x1b[0m" + after_cursor
            display = display.ljust(term_width + 8)[:term_width + 8]
        else:
            display = content.ljust(term_width)[:term_width]

        inside = prefix + display
        inside = inside + (" " * max(0, inner_content_width - len(inside)))
        return inside

    def display_content(self):
        """Display the current view of file contents"""
        cols = os.get_terminal_size().columns
        inner_content_width = max(0, cols - 2) + 2
        ui_min = max(0, self.current_line - self.cursor_rel_y)
        ui_max = min(ui_min + self.lines_to_show, len(self.file_lines))

        bounds = [ui_min, ui_max]
        snap = [0, len(self.file_lines)]

        ui_min, ui_max, scroll = adjust_range(
            self.current_line,
            bounds,
            snap,
            self.vertical_margin,
            self.is_moving_forward
        )

        self.cursor_rel_y -= scroll

        selection_range = None
        # if self.selection_anchor and (self.current_line != self.selection_anchor[0] or self.cursor_pos != self.selection_anchor[1]):
        #     selection_range = [self.selection_anchor, [self.current_line, self.cursor_pos]]
        #     selection_range = sorted(selection_range, key=lambda r: r[1])
        #     selection_range = sorted(selection_range, key=lambda r: r[0])

        # # Adjust start_line if we're near the end
        # if ui_max - ui_min < self.lines_to_show and ui_min > 0:
        #     ui_min = max(0, ui_max - self.lines_to_show)

        # Display file content
        for i in range(ui_min, ui_max):
            line_display = self.display_line(i, self.current_line, self.cursor_pos, selection_range)
            print(self.CLEAR_LINE + line_display)

        # Add empty placeholder lines if needed
        lines_shown = ui_max - ui_min
        for _ in range(self.lines_to_show - lines_shown):
            inner_content_width = max(0, cols - 6) + 2
            inside = "~" + (" " * max(0, inner_content_width - 1))
            print(self.CLEAR_LINE + inside)

    def display_status(self):
        """Display status separator and status line"""
        total_lines = len(self.file_lines)
        current_pos = self.current_line + 1

        cols = os.get_terminal_size().columns
        inner_content_width = max(0, cols - 2) + 2

        # Separator line
        print(self.CLEAR_LINE + self.H * inner_content_width)

        if self.mode != COMMAND_MODE:
            status = mode_status_text[self.mode]
            status += f"File: {self.filename} | Line {current_pos}/{total_lines} (Rel: {self.cursor_rel_y}) | Col {self.cursor_pos + 1}"
            if self.modified:
                status += " [modified]"
            status += " | Ctrl+S: save, Ctrl+Q: quit"

            if len(status) > inner_content_width:
                if inner_content_width >= 3:
                    status = status[:inner_content_width-3] + "..."
                else:
                    status = status[:inner_content_width]
            status = status + (" " * max(0, inner_content_width - len(status)))
        else:
            status = ":" + self.command_text

        total_ui_lines = self.lines_to_show + self.status_lines
        # Move cursor up to the start of the UI
        print(self.CLEAR_LINE + status)
        print(self.CURSOR_UP.format(total_ui_lines), end='')

    def execute_command(self):
        if self.command_text == 'write' or self.command_text == 'w':
            self.save_file()
        elif self.command_text == 'quit' or self.command_text == 'q':
            exit(0)
        elif self.command_text.isnumeric():
            line_idx = int(self.command_text) - 1
            if  0 <= line_idx < len(self.file_lines):
                self.current_line = line_idx
                self.cursor_pos = 0

        self.command_text = ""
        self.mode = NORMAL_MODE

    def save_file(self):
        """Save file to disk"""
        try:
            with open(self.filename, 'w', encoding='utf-8') as f:
                for line in self.file_lines:
                    f.write(line + '\n')
            self.modified = False
            return True
        except Exception:
            return False

    def run(self):
        """Main application loop"""
        if not self.load_file():
            return

        print(self.HIDE_CURSOR)  # Hide system cursor

        # Initial display
        self.display_content()
        self.display_status()

        print(self.SAVE_CURSOR, end='')

        try:
            while True:
                ch = self.get_char()
                if ch is None:
                    continue

                if ch == 'QUIT':  # Ctrl+Q
                    if self.modified:
                        # Could add confirmation here
                        pass
                    break
                elif ch == 'SAVE':  # Ctrl+S
                    self.save_file()
                elif ch == 'UP':
                    self.move_cursor_up()
                elif ch == 'DOWN':
                    self.move_cursor_down()
                elif ch == 'LEFT':
                    self.move_cursor_left()
                elif ch == 'RIGHT':
                    self.move_cursor_right()
                elif ch == 'BACKSPACE':
                    self.delete_char()
                elif ch == 'ENTER':
                    if self.mode == COMMAND_MODE:
                        self.execute_command()
                    else:
                        self.insert_newline()
                elif ch and len(ch) == 1 and ch.isprintable():
                    self.insert_char(ch)

                self.display_content()
                self.display_status()

        except KeyboardInterrupt:
            pass
        finally:
            # Clean up
            print(self.CLEAR_BELOW_CURSOR, end='')
            print(self.SHOW_CURSOR)  # Restore system cursor
            if self.modified:
                print("Warning: Unsaved changes!")


def main():
    if len(sys.argv) < 2:
        print("Usage: iedit filename [--lines nlines] [--vim]")

    filename = sys.argv[1]
    lines_to_show = 20
    vim_mode = False
    if '--lines' in sys.argv:
        idx = sys.argv.index('--lines')
        lines_to_show = int(sys.argv[idx + 1])
    if '--vim' in sys.argv:
        vim_mode = True

    if not os.path.exists(filename):
        print(f"File not found: {filename}")
        sys.exit(1)

    editor = InlineFileEditor(filename, lines_to_show, vim_mode)
    editor.run()

def get_last_match(matches):
    latest_match = None
    while True:
        try:
            latest_match = next(matches)
        except StopIteration:
            return latest_match

def strinsert(string, idx, substring):
    return f"{string[:idx]}{substring}{string[idx:]}"


def adjust_range(pos, bounds, snap, margin, is_increasing):
    lower_bound, upper_bound = bounds
    lower_snap, upper_snap = snap

    pos = max(pos, lower_bound)
    pos = min(pos, upper_bound)

    scroll = 0
    distance_to_start = pos - lower_bound
    distance_to_end = upper_bound - pos

    wiggle_start = lower_bound - lower_snap
    wiggle_end = upper_snap - upper_bound

    if is_increasing and distance_to_end < margin and wiggle_end > 0:
        scroll = min(margin - distance_to_end, wiggle_end)
    elif not is_increasing and distance_to_start < margin and wiggle_start > 0:
        scroll = max(-(margin - distance_to_start), -wiggle_start)

    return [lower_bound + scroll, upper_bound + scroll, scroll]

if __name__ == "__main__":
    main()
