use std::io::{self, Write, Stdout, BufReader};
use std::io::prelude::*;
use std::fs::{self, File};
use std::path::Path;

use termion::{color, cursor};
use termion::raw::{RawTerminal};

use crate::common::{Action, Color, Direction};
use crate::views::{Layer};
use crate::components::{logo, button, piano, slider};

const PALIT_ROOT: &str = "/usr/local/palit/";
const NUM_FOCII: usize = 3;
const NUM_PROJECTS: usize = 4;

// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path, mut collection: Vec<String>) -> io::Result<Vec<String>> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                collection.push(entry.file_name().into_string().unwrap());
            }
        }
    }
    Ok(collection)
}

// Store for heavy, static vars
pub struct Home {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    logo_asset: String,
    state: HomeState,
}

// Store for light, cloneable vars
#[derive(Clone, Debug)]
pub struct HomeState {
    motd: String,
    projects: Vec<String>,
    focus: usize,
    scroll_x: usize,
}

fn reduce(state: HomeState, action: Action) -> HomeState {
    let scroll_max = match state.projects.len()/4 {
        0 => 1,
        x => x+1,
    };
    let scroll_x = match action {
        Action::Left => {
            if state.scroll_x == 0 { scroll_max-1 } else {
                (state.scroll_x-1) % scroll_max
            }
        },
        Action::Right => (state.scroll_x+1) % scroll_max,
        _ => state.scroll_x,
    };
    let focus = match action {
	    Action::Up => if state.focus == 0 { NUM_FOCII-1 } else {
            (state.focus-1) % NUM_FOCII
        },
	    Action::Down => (state.focus+1) % NUM_FOCII,
	    _ => state.focus
    };
    HomeState {
        motd: state.motd.clone(),
        projects: state.projects.clone(),
        focus,
	    scroll_x,
    }
}

impl Home {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {

        // Load logo asset
        let asset_file = File::open("src/assets/logo.txt").unwrap();
        let mut buf_reader = BufReader::new(asset_file);
        let mut asset_str = String::new();
        buf_reader.read_to_string(&mut asset_str).unwrap();

        // Calculate center position
        let mut max_len: u16 = 0; 
        for line in asset_str.lines() {
            let len = line.len();
            if len as u16 > max_len {
                max_len = len as u16;
            }
        }

        let mut projects: Vec<String> = vec![];
        projects = visit_dirs(Path::new(PALIT_ROOT), projects).unwrap();

        // Initialize State
        let initial_state: HomeState = HomeState {
            motd: "It's Fun!".to_string(),
            projects,
            focus: 0,
	        scroll_x: 0,
        };

        Home {
            x: x + (width / 2) - (max_len / 2),
            y: y,
            width: width,
            height: height,
            logo_asset: asset_str,
            state: initial_state
        }
    }
}

impl Layer for Home {
    fn render(&self, mut out: RawTerminal<Stdout>) -> RawTerminal<Stdout> {

	// Logo
    out = logo::render(out, self.x, self.y);

	// New Button
	out = button::render(out, self.x + 10, self.y + 10, 17, "New Project");

	// Project Listing
	let mut col: [u16; 2] = [4,4];
        for (i, project) in self.state.projects.iter().enumerate() {
            if i >= self.state.scroll_x * NUM_PROJECTS && 
                i < (self.state.scroll_x+1) * NUM_PROJECTS {
                let j: u16 = (i % NUM_PROJECTS) as u16;
                let row: usize = (j % 2) as usize;
                write!(out, "{}", cursor::Goto(self.x+col[row] as u16, 
                    self.y+15+(row as u16 * 2))).unwrap();
                if self.state.focus == 0 { 
                    write!(out, "{}", color::Fg(color::Black)).unwrap();
                    match j {
                        0 => write!(out, "{}", color::Bg(color::Yellow)).unwrap(),
                        1 => write!(out, "{}", color::Bg(color::Magenta)).unwrap(),
                        2 => write!(out, "{}", color::Bg(color::Blue)).unwrap(),
                        3 => write!(out, "{}", color::Bg(color::Green)).unwrap(),
                        _ => write!(out, "{}", color::Bg(color::Reset)).unwrap(), 
                    }
                } else {
                    write!(out, "{}{}", 
                    color::Fg(color::White), 
                    color::Bg(color::Reset)
                    ).unwrap();
                }

                write!(out, " {} ", project).unwrap();
                col[row] += project.len() as u16 + 4;
            }
        }

        write!(out, "{}{}", color::Bg(color::Reset), color::Fg(color::Reset)).unwrap();

        out.flush().unwrap();

        out
    }

    fn dispatch(&mut self, action: Action) -> Action {
        self.state = reduce(self.state.clone(), action.clone());
        let num_projects: usize = self.state.projects.len();
        let num_choices: usize = {
            if (self.state.scroll_x+1)*NUM_PROJECTS < num_projects {
                NUM_PROJECTS
            } else { num_projects % NUM_PROJECTS }
        };
        match action {
            Action::SelectR => { Action::InputTitle }
            Action::SelectY => {
                Action::OpenProject(self.state.projects[self.state.scroll_x*NUM_PROJECTS].clone())
            },
            Action::SelectP => {
                if num_choices > 1 {
                    Action::OpenProject(self.state.projects[self.state.scroll_x*NUM_PROJECTS+1].clone())
                } else { Action::Noop }
            },
            Action::SelectB => {
                if num_choices > 2 {
                    Action::OpenProject(self.state.projects[self.state.scroll_x*NUM_PROJECTS+2].clone())
                } else { Action::Noop }
            },
            Action::SelectG => {
                if num_choices > 3 {
                    Action::OpenProject(self.state.projects[self.state.scroll_x*NUM_PROJECTS+3].clone())
                } else { Action::Noop }
            },
            Action::Up => { 
                if self.state.focus == 2 { Action::Up } else { Action::Noop }
            },
            Action::Down => {
                if self.state.focus == 0 { Action::Down } else { Action::Noop }
            },
            _ => Action::Noop,
        }
    }
    fn undo(&mut self) {
        self.state = self.state.clone()
    }
    fn redo(&mut self) {
        self.state = self.state.clone()
    }
    fn alpha(&self) -> bool {
        false
    }
}
