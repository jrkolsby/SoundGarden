use std::{iter};
use sample::{signal, Signal, Sample};
use xmltree::Element;
use libcommon::{Action, Param, param_map, note_to_hz};

use crate::core::{SF, Output, CHANNELS, SAMPLE_HZ};

// Standard Hammond drawbar.
const BAR_FREQS: [Param; 9] = [16., 5.+1./3., 8., 4., 2.+2./3., 2., 1.+3./5., 1.+1./3., 1.];

type SigGen = signal::Sine<signal::ConstHz>;

#[derive(Clone)]
pub struct Sig {
    note: u8,
    sig: SigGen,
    targetvol: f32,
    curvol: f32,
    baridx: usize,
}

pub struct Store {
    pub queue: Vec<Action>,
    pub sigs: Vec<Option<Sig>>,
    pub sample_rate: signal::Rate,
    pub bar_values: [Param; 9],
}

pub fn init() -> Store {
    Store {
        queue: vec![],
        sigs: iter::repeat(None).take(256).collect(),
        sample_rate: signal::rate(f64::from(SAMPLE_HZ)),
        bar_values: [0.25, 0.25, 0.25, 0.75, 0.5, 0., 0., 0., 0.],
    }
}

pub fn dispatch(store: &mut Store, action: Action) {
    match action {
        Action::NoteOn(note, vol) |
        Action::NoteOn(note, vol) => {
            let hz = note_to_hz(note);

            for (baridx, barfreq) in BAR_FREQS.iter().enumerate() {
                let idx = store.sigs.iter().position(|s| s.is_none());
                let idx = if let Some(idx) = idx { idx } else { return; }; // VOICE OVERFLOW
                let hz = store.sample_rate.const_hz((hz * 8. / barfreq) as f64);
                let s = Sig { sig: hz.sine(), note, targetvol: vol as f32, curvol: 0., baridx };
                store.sigs[idx] = Some(s);
            }
            // Only carry NoteOn/Off actions, NOT At(_, NoteOn)
            match action {
                Action::NoteOn(_, _) => store.queue.push(action),
                _ => {}
            };
        },
        Action::NoteOff(note) => {
            for i in store.sigs.iter_mut() {
                if let &mut Some(ref mut i) = i {
                    if i.note == note { i.targetvol = 0. }
                }
            }
            match action {
                Action::NoteOff(_) => store.queue.push(action),
                _ => {}
            };
        },
        Action::SetParam(ctrl, value) => {
            let idx = match ctrl.as_ref() {
                "16"    => 0,
                "5.3"   => 1,
                "8"     => 2,
                "4"     => 3,
                "2.6"   => 4,
                "2"     => 5,
                "1.6"   => 6,
                "1.3"   => 7,
                "1"     => 8,
                _ => return,
            };
            store.bar_values[idx] = value;
        }
        _ => {}
    }
}

pub fn read(doc: &mut Element) -> Option<Store> {
    let (_, params) = param_map(doc);
    let mut store = init();
    store.bar_values = [
        *params.get("16").unwrap_or(&0.1),
        *params.get("5.3").unwrap_or(&0.1),
        *params.get("8").unwrap_or(&0.1),
        *params.get("4").unwrap_or(&0.1),
        *params.get("2.6").unwrap_or(&0.1),
        *params.get("2").unwrap_or(&0.1),
        *params.get("1.6").unwrap_or(&0.1),
        *params.get("1.3").unwrap_or(&0.1),
        *params.get("1").unwrap_or(&0.1),
    ];
    Some(store)
}

pub fn compute(store: &mut Store) -> [Output; CHANNELS] {
    let mut z = 0f32;
    for sig in &mut store.sigs { 
        let mut remove = false;
        if let &mut Some(ref mut i) = sig {
            let barvalue = store.bar_values[i.baridx];
            if barvalue > 0.0 {
                let s = i.sig.next();
                z += s[0].mul_amp((i.curvol * barvalue) as f64) as f32;
            }

            // Quick and dirty volume envelope to avoid clicks. 
            if i.curvol != i.targetvol {
                if i.targetvol == 0. {
                    i.curvol -= 0.002;
                    if i.curvol <= 0. { remove = true; }
                } else {
                    i.curvol += 0.002;
                    if i.curvol >= i.targetvol { i.curvol = i.targetvol; }
                }
            }
        }
        if remove { *sig = None };
    }
    let z = z.min(0.999).max(-0.999);
    [z, z]
}

pub fn dispatch_requested(store: &mut Store) -> (
        Option<Vec<Action>>, // Actions for outputs
        Option<Vec<Action>>, // Actions for inputs
        Option<Vec<Action>> // Actions for client
    ) {
        let carry = store.queue.clone();
        store.queue.clear();
        (None, None, Some(carry.clone()))
}