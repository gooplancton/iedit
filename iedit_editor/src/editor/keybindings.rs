pub static HELP_POPUP_LINES: [&'static str; 14] = [
    "^g: go to",
    "^f: find forward",
    "^b: find backward",
    "^k: enter chord",
    "^z: undo",
    "^r: redo",
    "^y: yank (copy)",
    "^x: cut",
    "^p: paste",
    "^l: toggle selection",
    "^w: go to start of paragraph",
    "^e: go to end of paragraph",
    "^(: go to matching paren",
    "^n: go to next match",
];

pub static CHORDS_POPUP_LINES: [&'static str; 5] = [
    "l: line",
    "x: execute",
    "t: find char forward",
    "T: find char backward",
    "v: view",
];

pub static L_CHORD_POPUP_LINES: [&'static str; 4] = [
    "d: delete",
    "n: toggle numbers",
    "w: go to start",
    "e: go to end",
];

pub static X_CHORD_POPUP_LINES: [&'static str; 5] = [
    "x: auto (shebang)",
    "p: python3",
    "P: python",
    "n: node",
    "b: bash",
];

pub static T_CHORD_POPUP_LINES: [&'static str; 1] = [
    "press a key..."
];

pub static V_CHORD_POPUP_LINES: [&'static str; 1] = ["o: output/original"];
