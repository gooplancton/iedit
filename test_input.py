#!/usr/bin/env python3
import sys
import tty
import termios
import os

def main():
    print("Press Ctrl+Shift+Arrow keys to see their sequences")
    print("Press Ctrl+D to exit")
    print("-" * 50)

    fd = sys.stdin.fileno()
    old_settings = termios.tcgetattr(fd)

    try:
        tty.setraw(fd)

        while True:
            ch = sys.stdin.read(1)
            if ch == '' or ord(ch) == 4:  # EOF or Ctrl+D
                break

            bytes_data = ch.encode('latin-1')

            # If it's ESC, read the full escape sequence
            if bytes_data[0] == 0x1b:
                import select
                sequence = bytes_data
                while select.select([sys.stdin], [], [], 0.1)[0]:
                    more_ch = sys.stdin.read(1)
                    sequence += more_ch.encode('latin-1')

                hex_str = ' '.join(f'{b:02x}' for b in sequence)
                print(f"ESCAPE SEQUENCE: [{hex_str}] (length: {len(sequence)})", end='')

            else:
                hex_str = ' '.join(f'{b:02x}' for b in bytes_data)
                print(f"Regular key: [{hex_str}]", end='')
            print('\r\n')

    except KeyboardInterrupt:
        print("\nExiting...")
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)

if __name__ == "__main__":
    main()