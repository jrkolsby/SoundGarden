extern crate libcommon;
extern crate dsp;
extern crate libc;
extern crate sample;
extern crate hound;
extern crate chrono;

mod core;
mod midi;
mod synth;
mod tape;
mod chord;
mod arpeggio;
mod plugin;

use std::{iter, error};
use std::fs::{OpenOptions, File};
use std::os::unix::fs::OpenOptionsExt;
use std::io::prelude::*;
use std::collections::HashMap;
use std::borrow::BorrowMut;
use dsp::{NodeIndex, Frame, FromSample, Graph, Node, Sample, Walker};
use xmltree::Element;
use sample::signal;
use libcommon::{Action, Key, Document, read_document, param_map};

use crate::core::{event_loop, Module, Output, CHANNELS};
const MASTER_ROUTE_ID: u16 = 1;

fn add_module(
    id: u16,
    el: &mut Element,
    patch: &mut Graph<[Output; CHANNELS], Module>, 
    routes: &mut HashMap<u16, NodeIndex>, 
    operators: &mut HashMap<u16, NodeIndex>) {
        match &el.name[..] {
            "timeline" => {
                let mut anchors: Vec<NodeIndex> = vec![];

                // We need to load the tracks in order of their track ID
                // otherwise the patch will connect the wrong anchors
                let mut sorted_tracks = vec![];

                // Mutate el by removing track elements until none are left
                while let Some(store) = tape::read(el) {
                    sorted_tracks.push(store);
                }
                // Sort tracks
                sorted_tracks.sort_by(|a, b| a.track_id.partial_cmp(&b.track_id).unwrap());
                for track in sorted_tracks.drain(0..) {
                    let tape = patch.add_node(Module::Tape(track));
                    anchors.push(tape);
                    anchors.push(tape);
                }

                let operator = patch.add_node(Module::Operator(vec![], 
                    anchors.clone(), id.clone()
                ));
                // Because each track is stored as two anchors,
                // ... we need to make sure there is only one edge
                // ... to each track, otherwise actions will be 
                // ... dispatched two times. :^)
                for anchor in anchors.iter() {
                    if patch.find_connection(operator, *anchor).is_none() {
                        patch.add_connection(operator, *anchor);
                    }
                }
                operators.insert(id, operator);
            },
            "hammond" => {
                let store = match synth::read(el) {
                    Some(a) => a,
                    None => panic!("Invalid module {}", id)
                };
                let instrument = patch.add_node(Module::Synth(store));
                let operator = patch.add_node(Module::Operator(vec![], 
                    vec![instrument, instrument], id.clone()
                ));
                patch.add_connection(operator, instrument);
                operators.insert(id, operator);
            },
            "arpeggio" => {
                let store = match arpeggio::read(el) {
                    Some(a) => a,
                    None => panic!("Invalid module {}", id)
                };
                let inst = patch.add_node(Module::Arpeggio(store));
                let operator = patch.add_node(Module::Operator(vec![], 
                    vec![inst, inst], id.clone()
                ));
                patch.add_connection(operator, inst);
                operators.insert(id, operator);
            },
            "chord" => {
                let store = chord::read(el).unwrap();
                let inst = patch.add_node(Module::Chord(store));
                let operator = patch.add_node(Module::Operator(vec![], 
                    vec![inst, inst], id.clone()
                ));
                patch.add_connection(operator, inst);
                operators.insert(id, operator);
            },
            "keyboard" => {
                let (_, params) = param_map(el);
                let shift = *params.get("octave").unwrap_or(&3.0) as Key;
                let octave = patch.add_node(Module::Octave(vec![], shift));
                //let shift = patch.add_node(Module::Octave(vec![], 4));
                let operator = patch.add_node(Module::Operator(vec![], 
                    vec![octave, octave], id.clone()
                ));
                patch.add_connection(operator, octave);
                operators.insert(id, operator);
            },
            // This module should always be last in doc.modules or else 
            // operators and routes maps won't be completely filled
            "patch" => {
                while let Some(mut route_el) = el.take_child("route") {
                    let r_id = route_el.attributes.get("id").unwrap();
                    let _r_id = r_id.parse::<u16>().unwrap();
                    let route = patch.add_node(Module::Passthru(vec![]));
                    routes.insert(_r_id, route);
                    while let Some(input) = route_el.take_child("input") {
                        let m_id = input.attributes.get("module").unwrap();
                        let _m_id = m_id.parse::<u16>().unwrap();

                        let io_id = input.attributes.get("index").unwrap();
                        let _io_id = io_id.parse::<usize>().unwrap();
                        
                        let op_id = operators.get(&_m_id).unwrap();

                        let in_id = match &patch[*op_id] {
                            Module::Operator(_, anchors, _) => anchors[_io_id],
                            _ => panic!("No such input {}", io_id)
                        };
                        patch.add_connection(route, in_id);
                    }
                    while let Some(output) = route_el.take_child("output") {
                        let m_id = output.attributes.get("module").unwrap();
                        let _m_id = m_id.parse::<u16>().unwrap();
                        
                        let io_id = output.attributes.get("index").unwrap();
                        let _io_id = io_id.parse::<usize>().unwrap();

                        let op_id = operators.get(&_m_id).unwrap();

                        let out_id = match &patch[*op_id] {
                            Module::Operator(_, anchors, _) => anchors[_io_id],
                            _ => panic!("No such output {}", io_id)
                        };
                        patch.add_connection(out_id ,route);
                    }
                }
            },
            plugin => {
                // FIXME: use current directory
                let store = plugin::init(format!("./modules/{}.so", plugin));
                let plugin = patch.add_node(Module::Plugin(store));
                let operator = patch.add_node(Module::Operator(vec![], 
                    vec![plugin, plugin], id.clone()
                ));
                patch.add_connection(operator, plugin);
                operators.insert(id, operator);
            },
            name @ _ => { eprintln!("Unimplemented module {:?}", name)}
        }
    }

fn main() -> Result<(), Box<error::Error>> {

    // Blocked by pt-client reader
    println!("Waiting for pt-client...");

    // Configure pt-client IPC
    let mut ipc_client = match OpenOptions::new() 
        .write(true)
        .open("/tmp/pt-client") {
            Ok(a) => a,
            Err(_) => panic!("Could not open /tmp/pt-client")
        };

    let mut ipc_in = match OpenOptions::new()
        .custom_flags(libc::O_NONBLOCK)
        .read(true)
        .open("/tmp/pt-sound") {
            Ok(a) => a,
            Err(_) => panic!("Could not open /tmp/pt-sound")
        };

    // Construct our dsp graph.
    let mut graph = Graph::new();

    let mut operators: HashMap<u16, NodeIndex> = HashMap::new();
    let mut routes: HashMap<u16, NodeIndex> = HashMap::new();

    // Make a master route available without a project
    let master_node = graph.add_node(Module::Master);
    let master_route = graph.add_node(Module::Master);
    routes.insert(MASTER_ROUTE_ID, master_route);
    graph.add_connection(master_route, master_node);
    graph.set_master(Some(master_node));

    event_loop(ipc_in, ipc_client, graph, move |mut patch, a| { 
        // ROOT DISPATCH
        // n_id Node ID
        // r_id Route ID
        // a_id Anchor ID (Any input or output from a module)
        // op_id Module Operator ID (Dispatches to a cluster of nodes)
        // m_id Module ID (Key of operators)

        match a {
            Action::At(n_id, action) => {
                match *action {
                    Action::AddTrack(t_id) => {
                        if let Some(id) = operators.get(&n_id) {
                            let new_tape = patch.add_node(Module::Tape(tape::init(t_id)));
                            patch.add_connection(*id, new_tape);
                            match patch[*id] {
                                Module::Operator(_, ref mut anchors, _) => {
                                    anchors.push(new_tape); // INPUT
                                    anchors.push(new_tape); // OUTPUT
                                },
                                _ => {}
                            }
                        }
                    },
                    direct_action => {
                        if let Some(id) = operators.get(&n_id) {
                            patch[*id].dispatch(direct_action)
                        }
                    }
                }
            },
            Action::SetMeter(_, _) |
            Action::SetTempo(_) => {
                for (_, node) in operators.iter() {
                    patch[*node].dispatch(a.clone())
                }
            },
            Action::NoteOn(_,_) | Action::NoteOff(_) | Action::Octave(_) => {
                if let Some(id) = operators.get(&104) {
                    patch[*id].dispatch(a)
                }
            },
            Action::AddRoute(r_id) => {
                let route = patch.add_node(Module::Passthru(vec![]));
                routes.insert(r_id, route);
            },
            Action::PatchIn(n_id, a_id, r_id) => {
                if let Some(route) = routes.get(&r_id) {
                    match &patch[*operators.get(&n_id).unwrap()] {
                        Module::Operator(_, anchors, _) => {
                            let input = anchors[a_id as usize];
                            if let Err(e) = &patch.add_connection(*route, input) {
                                println!("CYCLE");
                            }
                        }
                        _ => {}
                    };
                }
            },
            Action::PatchOut(n_id, a_id, r_id) => {
                if let Some(route) = routes.get(&r_id) {
                    match &patch[*operators.get(&n_id).unwrap()] {
                        Module::Operator(_, anchors, _) => {
                            let output = anchors[a_id as usize];
                            if let Err(e) = &patch.add_connection(output, *route) {
                                println!("CYCLE");
                            }
                        }
                        _ => {}
                    };
                }
            },
            Action::DelPatch(n_id, a_id, input) => {
                match &patch[*operators.get(&n_id).unwrap()] {
                    Module::Operator(_, anchors, _) => {
                        let id = anchors[a_id as usize].clone();
                        for (_, route) in routes.iter() {
                            let edge = if input {
                                patch.find_connection(*route, id)
                            } else {
                                patch.find_connection(id, *route)
                            };
                            if let Some(e) = edge {
                                patch.remove_edge(e);
                            }
                        }
                    }
                    _ => {}
                }
            },
            Action::OpenProject(name) => {
                *patch = Graph::new();
                operators = HashMap::new();
                routes = HashMap::new();
                let mut doc = read_document(name);
                for (id, el) in doc.modules.iter_mut() {
                    add_module(*id, el, patch, &mut routes, &mut operators);
                }
                let root = patch.add_node(Module::Master);
                patch.set_master(Some(root));

                // Make sure we always have a master route 
                if !routes.contains_key(&MASTER_ROUTE_ID) {
                    routes.insert(MASTER_ROUTE_ID, 
                        patch.add_node(Module::Passthru(vec![])));
                }

                patch.add_connection(*routes.get(&MASTER_ROUTE_ID).unwrap(), root);
                eprintln!("Loaded {} Nodes", patch.node_count());
                eprintln!("Loaded {} Edges", patch.connection_count());
            },
            Action::AddModule(id, name) => {
                let mut new_el = Element::new(&name);
                new_el.attributes.insert("id".to_string(), id.to_string());
                add_module(id, &mut new_el, patch, &mut routes, &mut operators);
                eprintln!("Currently {} Nodes", patch.node_count());
                eprintln!("Currently {} Edges", patch.connection_count());
            }
            Action::DelModule(id) => {
                // Because removing a node from the graph will cause indicies to
                // ... shift, we're just going to lazily remove all edges on the
                // ... node cluster but leave the nodes there.
                if let Some(operator) = operators.remove(&id) {
                    let mut module_cluster = patch.outputs(operator);
                    while let Some(output_idx) = module_cluster.next_node(&patch) {
                        patch.remove_all_output_connections(output_idx);
                        patch.remove_all_input_connections(output_idx);
                    }
                }
            }
            Action::DelRoute(id) => {
                let route = routes.remove(&id).unwrap();
                patch.remove_all_output_connections(route);
                patch.remove_all_input_connections(route);
            }
            _ => { eprintln!("unimplemented action {:?}", a); }
        }
    })
}
