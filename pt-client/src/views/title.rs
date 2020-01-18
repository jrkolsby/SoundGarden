use std::io::{Write, Stdout};

use termion::{color, cursor};

use crate::common::{Screen, Action};
use crate::views::Layer;
use crate::components::{popup, casette, button};

pub struct Title {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    state: TitleState,
}

#[derive(Clone, Debug)]
pub struct TitleState {
    title: String,
    title_val: String,
    letter: u8,
}

const ASCII_MAX: u8 = 126;
const ASCII_MIN: u8 = 48;

fn reduce(state: TitleState, action: Action) -> TitleState {
    TitleState {
        title: state.title.clone(),
        title_val: match action {
            Action::Right => format!("{}{}", state.title_val, state.letter as char),
            Action::Left => state.title_val[..state.title_val.len()-1].to_string(),
            Action::SelectG => format!("{}.xml", state.title_val),
            Action::SelectY => "".to_string(),
            _ => state.title_val.clone(),
        },
        letter: match action {
            Action::Up => { if state.letter == ASCII_MAX { ASCII_MIN } else { state.letter+1 }},
            Action::Down => { if state.letter == ASCII_MIN { ASCII_MAX } else { state.letter-1 }},
            _ => state.letter.clone(),
        },
    }
}

impl Title {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {

        let mut path: String = "/usr/local/palit/".to_string();

        // Initialize State
        let initial_state: TitleState = TitleState {
            letter: ASCII_MIN,
            title_val: path,
            title: "What's it called?".to_string(),
        };

        Title {
            x: x,
            y: y,
            width: width,
            height: height,
            state: initial_state
        }
    }
}

impl Layer for Title {
    fn render(&self, out: &mut Screen, target: bool) {
        write!(out, "{}{}", color::Bg(color::Reset), color::Fg(color::Reset)).unwrap();

	    popup::render(out, self.x, self.y, self.width, self.height, &self.state.title);

        casette::render(out, self.x+2, self.y);

        write!(out, "{}\"{}{}\"", cursor::Goto(self.x+7, self.y+5), self.state.title_val, self.state.letter as char).unwrap();
        write!(out, "{} ▲ Letter ▼  ◀ Space ▶", cursor::Goto(self.x+7, self.y+16)).unwrap();

        write!(out, "{}{}{}  clear  ", cursor::Goto(self.x+24, self.y+18), color::Bg(color::Yellow), color::Fg(color::Black)).unwrap();
        write!(out, "{}{}{}  .xml  ", cursor::Goto(self.x+24, self.y+20), color::Bg(color::Green), color::Fg(color::Black)).unwrap();

        button::render(out, self.x+2, self.y+18, 20, "Create");

        write!(out, "{}{}", color::Bg(color::Reset), color::Fg(color::Reset)).unwrap();
    }

    fn dispatch(&mut self, action: Action) -> Action {
        self.state = reduce(self.state.clone(), action.clone());

        match action {
            Action::Back => Action::Cancel,
            Action::SelectR => Action::CreateProject(self.state.title_val.clone()),
            _ => Action::Noop
        }
    }
    fn undo(&mut self) {
        self.state = self.state.clone()
    }
    fn redo(&mut self) {
        self.state = self.state.clone()
    }
    fn alpha(&self) -> bool {
        true
    }
}
