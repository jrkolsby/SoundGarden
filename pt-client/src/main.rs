extern crate termion;

use std::io::{Write, stdout, stdin};

use termion::{clear, cursor};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode};

mod components; // NEED THIS IN MAIN FOR OTHER MODULES TO FIND IT!!!
mod common;
mod views;

use views::{Home, Timeline};

use common::{Action, Region, Asset, Track};

/*
mod utils;
mod components;
mod core;

use views::{Home};
use utils::{HomeState, read_document, write_document};
*/

fn main() -> std::io::Result<()> {

    // Configure stdin and raw_mode stdout
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let mut root: Home = Home::new();
    let mut timeline: Timeline = Timeline::new();

    write!(stdout, "{}{}", clear::All, cursor::Hide).unwrap();
    //stdout = root.render();
    stdout = timeline.render(stdout);
    stdout.flush().unwrap();

    // Loops until break
    for c in stdin.keys() {

        let action: Action = match c.unwrap() {
            Key::Char('q') => break,
            Key::Up => Action::Up,
            Key::Down => Action::Down,
            Key::Right => Action::SelectR,
            _ => Action::Noop,
        };

        write!(stdout, "{}", clear::All).unwrap();
        timeline.dispatch(action);

        //stdout = root.render(timeline.render(stdout));
        stdout = timeline.render(stdout);
    }

    // CLEAN UP
    write!(stdout, "{}{}{}", 
        clear::All, 
        cursor::Goto(1,1), 
        cursor::Show).unwrap();

    Ok(())
}