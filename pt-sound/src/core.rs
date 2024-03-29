extern crate sample;
extern crate portaudio;
extern crate hound;

use std::{iter, error};
use std::ffi::CString;
use std::fs::File;
use std::io::prelude::*;
use std::thread;
use std::sync::Arc;
use std::ops::DerefMut;
use libcommon::{Action, Key, Volume, Offset};

#[cfg(target_os = "linux")]
extern crate alsa;
#[cfg(target_os = "linux")]
use alsa::{seq, pcm, PollDescriptors};
#[cfg(target_os = "linux")]
use alsa::pcm::State;

use dsp::{sample::ToFrameSliceMut, NodeIndex, FromSample, Frame};
use dsp::{Outputs, Graph, Node, Sample, Walker};
use dsp::daggy::petgraph::Bfs;

#[cfg(target_os = "macos")]
use portaudio as pa;

use crate::midi::{open_midi_dev, read_midi_event, connect_midi_source_ports};
use crate::synth;
use crate::tape;
use crate::chord;
use crate::arpeggio;
use crate::plugin;

// SAMPLE FORMATS 
pub type Output = f32; // PORTAUDIO
pub type SF = i16; // ALSA

pub const CHANNELS: usize = 2;
pub const SAMPLE_HZ: f64 = 48_000.0;
pub const BUF_SIZE: usize = 24_000;
pub const BIT_RATE: usize = 16;
pub const FRAMES: u32 = 128;

const DEBUG_KEY_PERIOD: u16 = 24100;

#[derive(Debug, Clone)]
pub struct Note {
    pub id: u16,
    pub t_in: Offset,
    pub t_out: Offset,
    pub note: Key,
    pub vel: Volume,
}

#[cfg(target_os = "linux")]
pub fn open_audio_dev() -> Result<(alsa::PCM, u32), Box<error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 { 
        println!("Usage: 'cargo run --release CARD_NAME SAMPLE_RATE BUF_SIZE'");
        Err("No card name specified")?
    }
    let req_devname = format!("hw:{}", args[1]);
    let req_samplerate = args.get(2).map(|x| x.parse()).unwrap_or(Ok(48000))?;
    let req_bufsize = args.get(3).map(|x| x.parse()).unwrap_or(Ok(256))?; // A few ms latency by default, that should be nice 
    
    // Open the device
    let p = alsa::PCM::new(&req_devname, alsa::Direction::Playback, false)?;
    
    // Set hardware parameters
    {
        let hwp = pcm::HwParams::any(&p)?;
        hwp.set_channels(2)?;
        hwp.set_rate(req_samplerate, alsa::ValueOr::Nearest)?;
        hwp.set_format(pcm::Format::s16())?;
        hwp.set_access(pcm::Access::MMapInterleaved)?;
        hwp.set_buffer_size(req_bufsize)?;
        hwp.set_period_size(req_bufsize / 4, alsa::ValueOr::Nearest)?;
        p.hw_params(&hwp)?;
    }

    // Set software parameters
    let rate = {
        let hwp = p.hw_params_current()?;
        let swp = p.sw_params_current()?;
        let (bufsize, periodsize) = (hwp.get_buffer_size()?, hwp.get_period_size()?);
        swp.set_start_threshold(bufsize - periodsize)?;
        swp.set_avail_min(periodsize)?;
        p.sw_params(&swp)?;
        println!("Opened audio output {:?} with parameters: {:?}, {:?}", req_devname, hwp, swp);
        hwp.get_rate()?
    };

    Ok((p, rate))
}

#[cfg(target_os = "linux")]
fn set_buffer_size(p: &mut alsa::PCM, buf_size: i64) {
    let hwp = p.hw_params_current().unwrap();
    hwp.set_buffer_size(buf_size);
}

#[cfg(target_os = "linux")]
pub fn write_samples_direct(
    p: &alsa::PCM, 
    mmap: &mut alsa::direct::pcm::MmapPlayback<SF>, 
    synth: &mut Iterator<Item=SF>) -> Result<bool, Box<error::Error>>
{
    if mmap.avail() > 0 {
        mmap.write(&mut Box::new(synth));
    }

    match mmap.status().state() {
        State::Running => { return Ok(false); }, // All fine
        State::Prepared => { println!("Starting audio output stream"); p.start()? },
        State::XRun => { println!("Underrun in audio output stream!"); p.prepare()? },
        State::Suspended => { println!("Resuming audio output stream"); p.resume()? },
        n @ _ => Err(format!("Unexpected pcm state {:?}", n))?,
    }
    Ok(true) // Call us again, please, there might be more data to write
}

#[cfg(target_os = "linux")]
pub fn write_samples_io(
    p: &alsa::PCM, 
    io: &mut alsa::pcm::IO<SF>, 
    synth: &mut Iterator<Item=SF>) -> Result<bool, Box<error::Error>> 
{
    let avail = match p.avail_update() {
        Ok(n) => n,
        Err(e) => {
            println!("Recovering from {}", e);
            if let Some(errno) = e.errno() {
                p.recover(errno as std::os::raw::c_int, true)?;
            }
            p.avail_update()?
        }
    } as usize;

    if avail > 0 {
        io.mmap(avail, |buf| {
            for sample in buf.iter_mut() {
                *sample = synth.next().unwrap()
            };
	    buf.len() / 2 
        })?;
    }
    use alsa::pcm::State;
    match p.state() {
        State::Running => Ok(false), // All fine
        State::Prepared => { println!("Starting audio output stream"); p.start()?; Ok(true) },
        State::Suspended | State::XRun => Ok(true), // Recover from this in next round
        n @ _ => Err(format!("Unexpected pcm state {:?}", n))?,
    }
}

#[cfg(target_os = "macos")]
pub fn event_loop<F: 'static>(
        mut ipc_in: File, 
        mut ipc_client: File, 
        mut patch: Graph<[Output; CHANNELS], Module>, 
        mut dispatch_f: F) -> Result<(), Box<error::Error>> 
    where F: FnMut(&mut Graph<[Output; CHANNELS], Module>, Action) {

    // The callback we'll use to pass to the Stream. It will request audio from our dsp_graph.
    let callback = move |pa::OutputStreamCallbackArgs { buffer, time, .. }| {

        let ipc_actions: Vec<Action> = ipc_action(&ipc_in);

        match ipc_dispatch(ipc_actions, &mut patch, &mut dispatch_f) {
            Action::Exit => { return pa::Complete; },
            _ => {}
        }

        walk_dispatch(&ipc_client, &mut patch);

        let buffer: &mut [[Output; CHANNELS]] = buffer.to_frame_slice_mut().unwrap();
        dsp::slice::equilibrium(buffer);
        patch.audio_requested(buffer, SAMPLE_HZ);

        pa::Continue
    };

    // Construct PortAudio and the stream.
    let pa = pa::PortAudio::new()?;
    let settings = pa.default_output_stream_settings::<Output>(
        CHANNELS as i32, 
        SAMPLE_HZ, 
        FRAMES
    )?;
    let mut stream = pa.open_non_blocking_stream(settings, callback)?;
    stream.start()?;

    // Wait for our stream to finish.
    while let true = stream.is_active()? {
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    Ok(())
}

// Node types in our patch graph.
pub enum Module {
    // Exhibits default behavior of mixing inputs to its output
    Master,
    // A useful node which, when receiving an action, will dispatch it
    // ... to its neighbors
    Passthru(Vec<Action>),
    Octave(Vec<Action>, Key),
    // A hacky node which will dispatch NoteOn actions to its neighbors,
    // and every second or so will send all corresponding NoteOff actions.
    // Useful for debugging on OSX where keyup events aren't accessed.
    DebugKeys(Vec<Action>, Vec<Action>, u16),
    Operator(Vec<Action>, Vec<(NodeIndex)>, u16),  // Queue, Anchors, Module ID
    Synth(synth::Store),
    Tape(tape::Store),
    Chord(chord::Store),
    Arpeggio(arpeggio::Store),
    Plugin(plugin::Store),
}

impl Module {
    pub fn dispatch(&mut self, a: Action) {
        match *self {
            Module::Master => {}
            Module::Operator(ref mut queue, _, _) |
            Module::Passthru(ref mut queue) => { queue.push(a.clone()) }
            Module::DebugKeys(ref mut onqueue, _, _) => { onqueue.push(a.clone()); }
            Module::Synth(ref mut store) => synth::dispatch(store, a.clone()),
            Module::Tape(ref mut store) => tape::dispatch(store, a.clone()),
            Module::Chord(ref mut store) => chord::dispatch(store, a.clone()),
            Module::Octave(ref mut queue, ref mut n) => { 
                match a {
                    Action::NoteOn(_, _) | Action::NoteOff(_) => { queue.push(a.clone()); },
                    Action::Octave(up) => if up { *n = *n+1; } else { *n = if *n > 0 { *n-1 } else { 0 }; },
                    _ => (),
                }
            },
            Module::Arpeggio(ref mut store) => arpeggio::dispatch(store, a.clone()),
            Module::Plugin(ref mut store) => plugin::dispatch(store, a.clone()),
            _ => {}
        };
    }
    pub fn dispatch_requested(&mut self) -> (
            Option<Vec<Action>>, // Actions for outputs
            Option<Vec<Action>>, // Actions for inputs
            Option<Vec<Action>> // Actions for client
        ) {

        match *self {
            Module::Operator(ref mut queue, _, _) |
            Module::Passthru(ref mut queue) => {
                let carry = queue.clone();
                queue.clear();
                return (Some(carry), None, None)
            },
            Module::Octave(ref mut queue, ref mut dn) => {
                let mut client_carry = vec![];
                let mut carry = vec![];
                while let Some(note) = queue.pop() {
                    let shift: i8 = (12 * (*dn as i8 - 3)); // C3 is middle C (60)
                    let shifted_note = match note {
                        Action::NoteOn(n, v) => Action::NoteOn(
                            if shift > n as i8 { 0 } else { (n as i8 + shift) as u8 }, v),
                        Action::NoteOff(n) => Action::NoteOff(
                            if shift > n as i8 { 0 } else { (n as i8 + shift) as u8 }),
                        _ => Action::Noop,
                    };
                    carry.push(shifted_note.clone());
                    client_carry.push(shifted_note);
                }
                return (Some(carry), None, Some(client_carry))
            },
            Module::DebugKeys(ref mut onqueue, ref mut offqueue, ref mut timer) => {
                let carry = onqueue.clone();
                while let Some(note) = onqueue.pop() {
                    offqueue.push(match note {
                        Action::NoteOn(n, _) => Action::NoteOff(n),
                        _ => Action::Noop,
                    });
                }
                if *timer == 0 {
                    *timer = DEBUG_KEY_PERIOD;
                    return (Some(offqueue.clone()), None, None)
                } else {
                    return (Some(carry), None, None)
                }
            },
            Module::Tape(ref mut store) => tape::dispatch_requested(store),
            Module::Chord(ref mut store) => chord::dispatch_requested(store),
            Module::Arpeggio(ref mut store) => arpeggio::dispatch_requested(store),
            Module::Synth(ref mut store) => synth::dispatch_requested(store),
            Module::Plugin(ref mut store) => plugin::dispatch_requested(store),
            Module::Master => (None, None, None), // TODO: give master levels to client
            _ => (None, None, None)
        }
    }
}

impl Node<[Output; CHANNELS]> for Module {
    // Override the audio_requested method and compute PCM audio
    fn audio_requested(&mut self, buffer: &mut [[Output; CHANNELS]], sample_hz: f64) {
        match *self {
            Module::Master => (),
            Module::Synth(ref mut store) => {
                dsp::slice::map_in_place(buffer, |_| synth::compute(store));
            },
            Module::Tape(ref mut store) => {
                tape::compute_buf(store, buffer);
            },
            // Modules which aren't sound-producing can still implement audio_requested
            // ... to keep time, such as envelopes or arpeggiators
            Module::DebugKeys(_, _, ref mut timer) => {
                let dl = buffer.len() as u16;
                if *timer > dl { 
                    *timer = *timer - dl;
                } else {
                    *timer = 0;
                }
            },
            Module::Arpeggio(ref mut store) => {
                dsp::slice::map_in_place(buffer, |a| {
                    arpeggio::compute(store); a
                });
            },
            Module::Plugin(ref mut store) => {
                plugin::compute_buf(store, buffer);
            }
            _ => ()
        }
    }
}

// NOTE ABOUT EVENT LOOP TIMING
// assume buffer size of 512 frames, and a 48000Hz sample_rate,
// for each loop, we must write 512 frames to the audio device, 
// while the computation of these 512 frames might not take 
// 48000 / 512 seconds to calculate, that is the deadline, otherwise
// we get an audio underrun.
fn walk_dispatch(mut ipc_client: &File, patch: &mut Graph<[Output; CHANNELS], Module>) {
    // Nodes dispatch actions to its ins, outs, or to client. Midi signals
    // ... must travel opposite the direciton of audio in an acyclic graph
    let mut walk = patch.visit_order_rev();
    while let Some(n) = walk.next(&patch) {
        let (out_d, in_d, client_d) = patch[n].dispatch_requested();
        if let Some(mut out_a) = out_d {
            let mut outs = patch.outputs(n);
            while let Some(oid) = outs.next_node(&patch) {
                for a in out_a.iter() {
                    patch[oid].dispatch(a.clone());
                }
            }
        }
        if let Some(mut in_a) = in_d {
            let mut ins = patch.inputs(n);
            while let Some(iid) = ins.next_node(&patch) {
                for a in in_a.iter() {
                    patch[iid].dispatch(a.clone());
                }
            }
        }
        if let Some(client_a) = client_d {
            let mut ins = patch.inputs(n);
            let mut op_id = 0;
            'search: while let Some(oid) = ins.next_node(&patch) {
                match patch[oid] {
                    Module::Operator(_, _, id) => { op_id = id; break 'search; },
                    _ => {}
                }
            }
            for action in client_a.iter() {
                let filtered_direct = match action {
                    Action::Noop => None,
                    a => Some(Action::At(op_id, Box::new(a.to_owned()))),
                };
                if let Some(a) = filtered_direct {
                    ipc_client.write(a.to_string().as_bytes());
                }
            }
        }
    }
}

fn ipc_dispatch<F: 'static>(
        ipc_actions: Vec<Action>, 
        patch: &mut Graph<[Output; CHANNELS], Module>,
        root_dispatch: &mut F) -> Action 

    where F: FnMut(&mut Graph<[Output; CHANNELS], Module>, Action) {

    for action in ipc_actions.iter() {
        match action {
            Action::Exit => { return Action::Exit; },
            // Pass any other action to root
            a => { 
                root_dispatch(patch, a.clone()); 
            }
        };
    }
    Action::Noop
}

fn ipc_action(mut ipc_in: &File) -> Vec<Action> {
    let mut buf: String = String::new();
    ipc_in.read_to_string(&mut buf);
    let mut ipc_iter = buf.split(" ");

    let mut events: Vec<Action> = Vec::new();

    while let Some(action_raw) = ipc_iter.next() {
        match action_raw.parse::<Action>() {
            Ok(Action::Noop) => (),
            Ok(a) => { events.push(a) },
            Err(r) => (),
        };
    };

    events
}

#[cfg(target_os = "linux")]
pub fn event_loop<F: 'static>(
        mut ipc_in: File, 
        mut ipc_client: File, 
        mut patch: Graph<[Output; CHANNELS], Module>, 
        mut dispatch_f: F) -> Result<(), Box<error::Error>> 
    where F: FnMut(&mut Graph<[Output; CHANNELS], Module>, Action) {
    
    // Get audio devices
    let (audio_dev, rate) = open_audio_dev()?;

    // Get midi devices
    let midi_dev = open_midi_dev()?;
    let mut midi_input = midi_dev.input();

    // Create an array of file descriptors to poll
    let mut fds = audio_dev.get()?;
    fds.append(&mut (&midi_dev, Some(alsa::Direction::Capture)).get()?); 
    
    // Use direct-mode memory mapping for minimum overhead
    let mut mmap = audio_dev.direct_mmap_playback::<SF>();
    
    // if direct-mode unavailable, use mmap emulation instead
    let mut io = if mmap.is_err() {
        Some(audio_dev.io_i16()?)
    } else { None };

    loop {

        let ipc_actions: Vec<Action> = ipc_action(&ipc_in);

        match ipc_dispatch(ipc_actions, &mut patch, &mut dispatch_f) {
            Action::Exit => { return Ok(()) },
            _ => {}
        }

        walk_dispatch(&ipc_client, &mut patch);

        let buffer: &mut [[Output; CHANNELS]] = &mut [[0.0; CHANNELS]; FRAMES as usize];

        dsp::slice::equilibrium(buffer);

        patch.audio_requested(buffer, rate as f64);

        // TODO: float->int sample conversion
        let mut buf_iter = buffer.iter().map(|a| (a[0]*500.0) as i16);

        if let Ok(ref mut mmap) = mmap {
            if write_samples_direct(&audio_dev, mmap, &mut buf_iter)? { continue; }
        } else if let Some(ref mut io) = io {
            if write_samples_io(&audio_dev, io, &mut buf_iter)? { continue; }
        }

        //if read_midi_event(&mut midi_input, &mut root.synths[0])? { continue; }

        // Nothing to do, let's sleep until woken up by the kernel.
        alsa::poll::poll(&mut fds, 100)?;
    }
}
