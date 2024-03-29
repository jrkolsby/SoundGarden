use std::io::{Write, Stdout};
use termion::{cursor};
use xmltree::Element;
use libcommon::Action;

use crate::common::Screen;
use crate::views::{Layer};
use crate::components::{popup, keyboard};

pub struct Help {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    state: HelpState,
    history: Vec<HelpState>,
}

#[derive(Clone, Debug)]
pub struct HelpState {
    title: String,
    active: Vec<Action>
}

fn reduce(state: HelpState, action: Action) -> HelpState {
    HelpState {
        title: state.title.clone(),
        active: match action {
            Action::NoteOn(_, _) => {
                let mut new_active = state.active.clone();
                new_active.push(action);
                new_active
            }
            _ => state.active.clone()
        },
    }
}

impl Help {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        // Initialize State
        let initial_state: HelpState = HelpState {
            title: "KEYBOARD".to_string(),
            active: vec![]
        };

        Help {
            x: x,
            y: y,
            width: width,
            height: height,
            history: vec![],
            state: initial_state
        }
    }
}

impl Layer for Help {
    fn render(&self, out: &mut Screen, target: bool) {
	    popup::render(out, self.x, self.y, self.width, self.height, &self.state.title);
        keyboard::render(out, &self.state.active, self.x + 4, self.y + 4);

        write!(out, "{}Press Q to go back", cursor::Goto(self.x + 2, self.y + 2)).unwrap();
    }

    fn dispatch(&mut self, action: Action) -> Action {
        self.state = reduce(self.state.clone(), action.clone());
        match action {
            Action::Back |
            Action::SelectR => Action::Cancel,
            _ => Action::Noop
        }
    }
    fn alpha(&self) -> bool { true }
    fn save(&self) -> Option<Element> { None }
}
