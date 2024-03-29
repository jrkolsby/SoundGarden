use termion::cursor;
use std::io::Write;
use crate::common::Screen;

const FONT: &str = r#"
▗▛▙    █▀▜▖█  ▗▛▜▖   █▀▙  ▐▌█▀▀▘   █▀▀▘▗▛▖▗▛▀▘▗▖▖
█ ▐▌▟▜▌█▄▟▘█▄ █   ▟▜▖█ ▐▌▗▟▌█▄▖ ▟▀▖█▄▖ ▟▙ █ ▄▖█▟ 
█▀▜▌█▐▌█ ▐▌█▐▌▜▖▗▖▙ ▖█ ▟▘█▐▌█   █▀▘█   ▐▌ ▜▖▐▌▟▄▖
▀ ▝▘▝▘▀▀▀▀ ▀▀  ▀▀ ▝▀ ▀▀▘ ▝▘▀▀▀▀▘▝▀ ▀   ▝▘  ▀▀▘▚▟▘
█ ▐▌█  ▀█▀▗▖  ▐▌ ▗▖█ ▟▘█  █  ▜▌█▖▟▌    █▖▐▌   ▟▀▜▖   
█▄▟▌█▄  █ ▄▖  ▐▌ ▗▖█▟▘ █▗▘█  ▐▌███▌█▚▜▖██▟▌▖▄▖█ ▐▌▗▄ 
█ ▐▌█▐▌ █ ▐▌▄ ▐▌ ▐▌█▜▙ █▙ █  ▐▌█▝▐▌█▐▐▌█▝█▌▐▌██ ▐▌█▐▌
▀ ▝▘▀▝▘▀▀▀▝▘▝▀▘ ▚▟▘▀ ▀▘▀▝▘▀▀▀▝▘▀ ▝▘▀▝▝▘▀ ▝▘▝▘▀ ▀▀ ▝▀ 
█▀▜▖   ▟▀▜▖   █▀▜▖   ▟▀▜▖   ▀█▀▐▌ █ ▐▌   █ ▐▌   
█ ▐▌▚▛▙█ ▐▌▟▜▞█ ▟▌▚█▚▜▄▄ ▟▀  █ ▜▛▘█ ▐▌█▐▌█▖▟▌█ ▐
█▀▀ ▐▙▛█▝█▘▜▟▌█▜▙ ▐▌ ▄ ▐▌▝▜▖ █ ▐▌ █ ▐▌█▐▌▝█▛ ▐▙▛
▀   ▐▌ ▝▀▘▘ ▐▌▀ ▀▘▝▘ ▝▀▀ ▀▀  ▀  ▀ ▝▀▀ ▝▘▀ ▝   ▀ 
█ ▐▌    █▖▟▌   █ █   ▀▀█▌    
█▟▟▌█ ▐▌▝█▛ █▖▞▜▄▛▜▖▞ ▟▛ ▀▜▛ 
█▛█▌█▐▐▌▟▛█▖▝█▖ █  ▜▘▟▛  ▗▛  
▝ ▝ ▝▘▀ ▀ ▝▘▀▝▀ ▀ ▗▌ ▀▀▀▘▀▀▀ 
▗▀▙ ▗█ ▟▀▜▖▝▀█▘ ▟█ █▀▀ ▗▛▀ █▀▜▌▟▀▚ ▟▀▜▖    
█ ▐▌ █  ▄█▘ ▟▙ ▟▘█ ▀▀▜▖█▄▄  ▗▛ ▜▙▞ ▜▄▟▌    
▜▖▐▘ █ ▟▛▘ ▄ ▐▌▀▀█▘▄ ▐▌█ ▐▌ █  ▌▝▜▌  ▟▘    
 ▀▘ ▀▀▀▀▀▀▘▝▀▀   ▀ ▝▀▀ ▝▀▀  ▀  ▝▀▀ ▝▀▘ ▀   
"#;

pub fn render(out: &mut Screen, x: u16, y: u16, text: String) {
    let font: Vec<Vec<char>> = FONT.lines().map(|l| 
        l.chars().collect()
    ).collect();
    let mut current_x = x;
    for c in text.chars() {
        let (_x,_y,_w): (usize,usize,usize) = match c {
            ' ' => (41,17,2),
            '.' => (39,17,2),
            '9' => (35,17,4),
            '8' => (31,17,4),
            '7' => (27,17,4),
            '6' => (23,17,4),
            '5' => (19,17,4),
            '4' => (15,17,4),
            '3' => (11,17,4),
            '2' => (7, 17,4),
            '1' => (4, 17,3),
            '0' => (0, 17,4),
            'z' => (25,13,3),
            'Z' => (21,13,4),
            'y' => (18,13,3),
            'Y' => (15,13,3),
            'x' => (12,13,3),
            'X' => (8, 13,4),
            'w' => (4, 13,4),
            'W' => (0, 13,4),
            'v' => (45,9, 3),
            'V' => (41,9, 4),
            'u' => (38,9, 3),
            'U' => (34,9, 4),
            't' => (31,9, 3),
            'T' => (28,9, 3),
            's' => (25,9, 3),
            'S' => (21,9, 4),
            'r' => (18,9, 3),
            'R' => (14,9, 4),
            'q' => (11,9, 3),
            'Q' => (7, 9, 4),
            'p' => (4, 9, 3),
            'P' => (0, 9, 4),
            'o' => (50,5, 3),
            'O' => (46,5, 4),
            'n' => (43,5, 3),
            'N' => (39,5, 4),
            'm' => (35,5, 4),
            'M' => (31,5, 4),
            'l' => (29,5, 2),
            'L' => (26,5, 3),
            'k' => (23,5, 3),
            'K' => (19,5, 4),
            'j' => (16,5, 3),
            'J' => (12,5, 4),
            'i' => (10,5, 2),
            'I' => (7, 5, 3),
            'h' => (4, 5, 3),
            'H' => (0, 5, 4),
            'g' => (46,1, 3),
            'G' => (42,1, 4),
            'f' => (39,1, 3),
            'F' => (35,1, 4),
            'e' => (32,1, 3),
            'E' => (28,1, 4),
            'd' => (25,1, 3),
            'D' => (21,1, 4),
            'c' => (18,1, 3),
            'C' => (14,1, 4),
            'b' => (11,1, 3),
            'B' => (7, 1, 4),
            'a' => (4, 1, 3),
            'A' => (0, 1, 4),
            _ => (0,0,0),
        };
        for dy in _y.._y+4 {
            write!(out, "{}{}", 
                cursor::Goto(current_x, 1 + y + (dy-_y) as u16), 
                font[dy][_x..(_x + _w)].iter().cloned().collect::<String>()
            ).unwrap();
        }
        current_x += _w as u16;
    };
}
