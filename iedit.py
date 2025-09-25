#!/usr/bin/env python3
"""
Inline File Editor - A scrollable file editor with direct cursor navigation
Usage: python editor.py <filename>
Controls:
  ↑/↓ - move cursor up/down
  ←/→ - move cursor left/right
  Ctrl+S - save file
  Ctrl+Q - quit without saving
  Backspace - delete character before cursor
  Any printable character - insert at cursor
"""

import sys
import os
import termios
import tty
import select

class InlineFileEditor:
    def __init__(self, filename, lines_to_show=10):
        self.filename = filename
        self.lines_to_show = lines_to_show
        self.current_line = 0
        self.file_lines = []
        self.status_lines = 2  # Separator + status text
        self.vertical_margin = 5

        # Cursor position within the current line
        self.cursor_pos = 0
        self.modified = False

        # Track previous state for minimal redraws
        self.prev_line = 0
        self.prev_cursor_pos = 0
        self.prev_start_line = 0

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

    def get_char(self):
        """Get a single character from stdin without requiring Enter"""
        fd = sys.stdin.fileno()
        old_settings = termios.tcgetattr(fd)
        try:
            tty.setcbreak(fd)
            if select.select([sys.stdin], [], [], 0.1)[0]:
                ch = sys.stdin.read(1)
                if ch == '\033':  # Escape sequence
                    next_chars = sys.stdin.read(2)
                    if next_chars == '[A':
                        return 'UP'
                    elif next_chars == '[B':
                        return 'DOWN'
                    elif next_chars == '[C':
                        return 'RIGHT'
                    elif next_chars == '[D':
                        return 'LEFT'
                elif ch == '\x11':  # Ctrl+Q
                    return 'QUIT'
                elif ch == '\x13':  # Ctrl+S
                    return 'SAVE'
                elif ch == '\x7f':  # Backspace
                    return 'BACKSPACE'
                elif ch == '\r' or ch == '\n':  # Enter
                    return 'ENTER'
                else:
                    return ch
            return None
        finally:
            termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)

    def move_cursor_up(self):
        """Move cursor up one line"""
        if self.current_line > 0:
            self.current_line -= 1
            # Keep cursor position within bounds of new line
            self.cursor_pos = min(self.cursor_pos, len(self.file_lines[self.current_line]))

    def move_cursor_down(self):
        """Move cursor down one line"""
        if self.current_line < len(self.file_lines) - 1:
            self.current_line += 1
            # Keep cursor position within bounds of new line
            self.cursor_pos = min(self.cursor_pos, len(self.file_lines[self.current_line]))

    def move_cursor_left(self):
        """Move cursor left one character"""
        if self.cursor_pos > 0:
            self.cursor_pos -= 1

    def move_cursor_right(self):
        """Move cursor right one character"""
        current_line_content = self.file_lines[self.current_line]
        if self.cursor_pos < len(current_line_content):
            self.cursor_pos += 1

    def insert_char(self, ch):
        """Insert a character at cursor position"""
        current_line = self.file_lines[self.current_line]
        new_line = current_line[:self.cursor_pos] + ch + current_line[self.cursor_pos:]
        self.file_lines[self.current_line] = new_line
        self.cursor_pos += 1
        self.modified = True

    def delete_char(self):
        """Delete character before cursor"""
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

    def display_line(self, line_idx, current_line, cursor_pos):
        """Display a single line with proper formatting"""
        cols = os.get_terminal_size().columns
        inner_content_width = max(0, cols - 2) + 2
        line_num = line_idx + 1
        prefix = f"{line_num:4d}: "
        term_width = inner_content_width - len(prefix)

        line_text = self.file_lines[line_idx]
        offset = max(self.cursor_pos - term_width + 1, 0)
        content = line_text[offset:offset + term_width]

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
        start_line = max(0, self.current_line - self.lines_to_show // 2)
        end_line = min(start_line + self.lines_to_show, len(self.file_lines))

        # Adjust start_line if we're near the end
        if end_line - start_line < self.lines_to_show and start_line > 0:
            start_line = max(0, end_line - self.lines_to_show)

        # Display file content
        for i in range(start_line, end_line):
            line_display = self.display_line(i, self.current_line, self.cursor_pos)
            print(self.CLEAR_LINE + line_display)

        # Add empty placeholder lines if needed
        lines_shown = end_line - start_line
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

        # Status text
        status = f"File: {self.filename} | Line {current_pos}/{total_lines} | Col {self.cursor_pos + 1}"
        if self.modified:
            status += " [modified]"
        status += " | Ctrl+S: save, Ctrl+Q: quit"

        if len(status) > inner_content_width:
            if inner_content_width >= 3:
                status = status[:inner_content_width-3] + "..."
            else:
                status = status[:inner_content_width]
        status = status + (" " * max(0, inner_content_width - len(status)))
        print(self.CLEAR_LINE + status)

        total_ui_lines = self.lines_to_show + self.status_lines
        # Move cursor up to the start of the UI
        print(self.CURSOR_UP.format(total_ui_lines), end='')

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
    filename = sys.argv[1]
    lines_to_show = 20
    if len(sys.argv) == 4 and sys.argv[2] == "--lines":
        lines_to_show = int(sys.argv[3])

    if not os.path.exists(filename):
        print(f"File not found: {filename}")
        sys.exit(1)

    editor = InlineFileEditor(filename, lines_to_show)
    editor.run()

if __name__ == "__main__":
    main()
