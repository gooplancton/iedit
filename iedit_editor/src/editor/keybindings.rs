pub static HELP_POPUP_LINES: [&'static str; 10] = [
    "^k      │ enter chord",
    "^g      │ go to",
    "^f (^b) │ forward (backward) search",
    "^z (^r) │ undo (redo)",
    "^y (^x) │ copy (cut)",
    "^p      │ paste",
    "^o (^l) │ go to previous (next) jump",
    "^w (^e) │ go to start (end) of paragraph",
    "^(      │ go to matching paren",
    "^n      │ go to next match",
];

pub static CHORDS_POPUP_LINES: [&'static str; 6] = [
    "l │ line",
    "x │ execute",
    "s │ selection",
    "t │ find char forward",
    "T │ find char backward",
    "v │ view",
];

pub static L_CHORD_POPUP_LINES: [&'static str; 4] = [
    "d │ delete",
    "n │ toggle numbers",
    "w │ go to start",
    "e │ go to end",
];

pub static X_CHORD_POPUP_LINES: [&'static str; 6] = [
    "x │ auto",
    "? | manual",
    "p │ python3",
    "P │ python",
    "n │ node",
    "b │ bash",
];

pub static T_CHORD_POPUP_LINES: [&'static str; 1] = ["press a key..."];

pub static S_CHORD_POPUP_LINES: [&'static str; 1] = ["l │ lock/unlock selection"];

pub static V_CHORD_POPUP_LINES: [&'static str; 1] = ["o │ output/original"];
