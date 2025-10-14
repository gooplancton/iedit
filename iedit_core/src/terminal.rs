#![allow(dead_code, unused)]

use std::io::stdout;

use termion::{cursor::DetectCursorPos, raw::IntoRawMode};

pub static CURSOR_UP1: &str = "\x1b[1A";
pub static CURSOR_DOWN1: &str = "\x1b[1B";
pub static CURSOR_RIGHT1: &str = "\x1b[1C";
pub static CURSOR_LEFT1: &str = "\x1b[1D";
pub static CURSOR_TO_LINE1: &str = "\x1b[1;1H";
pub static CLEAR_LINE: &str = "\x1b[2K";
pub static CLEAR_BELOW_CURSOR: &str = "\x1b[J";
pub static CURSOR_TO_COL1: &str = "\r";
pub static SAVE_CURSOR: &str = "\x1b[s";
pub static RESTORE_CURSOR: &str = "\x1b[u";
pub static HIDE_CURSOR: &str = "\x1b[?25l";
pub static SHOW_CURSOR: &str = "\x1b[?25h";
pub static CLEAR_SCREEN: &str = "\x1b[2J";
pub static HIGHLIGHT_START: &str = "\x1b[7m";
pub static HIGHLIGHT_END: &str = "\x1b[0m";
pub static H_BAR: &str = "─";
pub static V_BAR: char = '│';
