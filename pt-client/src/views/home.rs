use std::io::{self, Write, Stdout, BufReader};
use std::io::prelude::*;
use termion::{color, cursor};
use xmltree::Element;
use libcommon::{Action, Anchor};

use crate::common::Screen;
use crate::common::{get_files, PALIT_PROJECTS};
use crate::views::Layer;
use crate::components::{logo, button, bigtext};

const NUM_FOCII: usize = 1;
const NUM_PROJECTS: usize = 4;
const SIZE: (u16, u16) = (34, 10);

// Store for heavy, static vars
pub struct Home {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    state: HomeState,
}

// Store for light, cloneable vars
#[derive(Clone, Debug)]
pub struct HomeState {
    motd: String,
    projects: Vec<String>,
    scroll_x: usize,
}

fn reduce(state: HomeState, action: Action) -> HomeState {
    let scroll_max = (match state.projects.len() { 0 => 0, x => x - 1 }) / 4 + 1;
    HomeState {
        motd: state.motd.clone(),
        projects: match action.clone() {
            Action::ShowProject(title, _) => {
                let mut new_projects = state.projects.clone();
                new_projects.push(title);
                new_projects
            },
            _ => state.projects.clone(),
        },
	    scroll_x: match action {
            Action::Left => {
                if state.scroll_x == 0 { scroll_max - 1 } 
                else { (state.scroll_x - 1) % scroll_max }
            },
            Action::Right => (state.scroll_x + 1) % scroll_max,
            _ => state.scroll_x,
        },
    }
}

impl Home {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {

        let projects = get_files(PALIT_PROJECTS, "xml", vec![]).unwrap();

        // Initialize State
        let initial_state: HomeState = HomeState {
            motd: "It's Fun!".to_string(),
            projects,
	        scroll_x: 0,
        };

        Home {
            x: x + (width / 2) - (SIZE.0 / 2),
            y: y + (height / 2) - (SIZE.1 / 2),
            width: width,
            height: height,
            state: initial_state
        }
    }
}

impl Layer for Home {
    fn render(&self, out: &mut Screen, target: bool) {

        bigtext::render(out, self.x, self.y, "Song Garden".to_string());

        // Project Listing
        let mut col: [u16; 2] = [4,4];
        for (i, project) in self.state.projects.iter().enumerate() {
            if i >= self.state.scroll_x * NUM_PROJECTS && 
                i < (self.state.scroll_x+1) * NUM_PROJECTS {
                let j: u16 = (i % NUM_PROJECTS) as u16;
                let row: usize = (j % 2) as usize;
                write!(out, "{}", cursor::Goto(self.x+col[row] as u16, 
                    self.y + 6 + (row as u16 * 2))).unwrap();
                write!(out, "{}", color::Fg(color::Black)).unwrap();
                match j {
                    0 => write!(out, "{}", color::Bg(color::Yellow)).unwrap(),
                    1 => write!(out, "{}", color::Bg(color::Blue)).unwrap(),
                    2 => write!(out, "{}", color::Bg(color::Green)).unwrap(),
                    3 => write!(out, "{}", color::Bg(color::Magenta)).unwrap(),
                    _ => write!(out, "{}", color::Bg(color::Reset)).unwrap(), 
                }
                write!(out, " {} ", project).unwrap();
                col[row] += project.len() as u16 + 4;
            }
        }

        write!(out, "{}{}", color::Bg(color::Reset), color::Fg(color::Reset)).unwrap();
    }

    fn dispatch(&mut self, action: Action) -> Action {
        self.state = reduce(self.state.clone(), action.clone());
        let num_projects: usize = self.state.projects.len();
        let mut num_choices = num_projects - (self.state.scroll_x * NUM_PROJECTS);
        num_choices = if num_choices > 4 { 4 } else { num_choices };
        match action {
            Action::Back => Action::Exit,
            Action::SelectY => {
                Action::OpenProject(self.state.projects[self.state.scroll_x * NUM_PROJECTS].clone())
            },
            Action::SelectB => {
                if num_choices > 1 {
                    Action::OpenProject(self.state.projects[self.state.scroll_x * NUM_PROJECTS + 1].clone())
                } else { Action::Noop }
            },
            Action::SelectG => {
                if num_choices > 2 {
                    Action::OpenProject(self.state.projects[self.state.scroll_x * NUM_PROJECTS + 2].clone())
                } else { Action::Noop }
            },
            Action::SelectP => {
                if num_choices > 3 {
                    Action::OpenProject(self.state.projects[self.state.scroll_x * NUM_PROJECTS + 3].clone())
                } else { Action::Noop }
            },
            a @ Action::Up |
            a @ Action::Down => a,
            _ => Action::Noop,
        }
    }
    fn alpha(&self) -> bool { false }
    fn save(&self) -> Option<Element> { None }
}
