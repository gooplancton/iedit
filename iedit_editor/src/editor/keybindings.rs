pub static HELP_POPUP_LINES: [&str; 11] = [
    "Ctrl-k      │ enter chord",
    "Ctrl-g      │ go to",
    "Ctrl-e      │ open command prompt",
    "Ctrl-f (-b) │ forward (backward) search",
    "Ctrl-z (-r) │ undo (redo)",
    "Ctrl-y (-x) │ copy (cut)",
    "Ctrl-p      │ paste",
    "Ctrl-u (-d) │ go up (down) a page",
    "Alt-o  (-i) │ go to previous (next) jump",
    "Alt-n  (-m) │ go to next (previous) match",
    "Alt-p       │ go to matching paren",
];

pub static CHORDS_POPUP_LINES: [&str; 6] = [
    "l │ line",
    "x │ execute",
    "s │ selection",
    "t │ find char forward",
    "T │ find char backward",
    "v │ view",
];

pub static L_CHORD_POPUP_LINES: [&str; 4] = [
    "d │ delete",
    "n │ toggle numbers",
    "w │ go to start",
    "e │ go to end",
];

pub static X_CHORD_POPUP_LINES: [&str; 6] = [
    "x │ auto",
    "? │ manual",
    "p │ python3",
    "P │ python",
    "n │ node",
    "b │ bash",
];

pub static T_CHORD_POPUP_LINES: [&str; 1] = ["press a key..."];

pub static S_CHORD_POPUP_LINES: [&str; 1] = ["l │ lock/unlock selection"];

pub static V_CHORD_POPUP_LINES: [&str; 1] = ["o │ output/original"];
