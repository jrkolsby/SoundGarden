use termion::cursor;
use std::io::Write;
use crate::common::Screen;

const LOGO: &str = r#"
d8888b.  .d8b.  db      d888888b d888888b
88  `8D d8' `8b 88      `~~88~'  `~~88~~'
88oodD' 88ooo88 88         88       88   
88~~~   88~~~88 88         88       88   
88      88   88 88booo.   .88.      88   
YP      YP   YP Y88888P Y888888P    YP   
"#;

pub fn render(out: &mut Screen, x: u16, y: u16) {
    for (i, line) in LOGO.lines().enumerate() {
        write!(out, "{}{}",
            cursor::Goto(x, (i as u16)+y+1),
            line).unwrap();
    };
}
