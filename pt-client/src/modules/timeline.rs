use std::convert::TryInto;
use std::collections::HashMap;

use xmltree::Element;

use crate::views::TimelineState;
use crate::views::Timeline;
use crate::modules::{param_map, mark_map};
use crate::common::{Region, Track, Asset};

pub fn write(state: TimelineState) -> Element {
    Element::new("param")
}

pub fn read(doc: Element) -> TimelineState {

    let (doc, params) = param_map(doc);
    let (mut doc, marks) = mark_map(doc);

    let mut state = TimelineState {

        tempo: *params.get("bpm").unwrap_or(&127) as u16,
        meter_beat: *params.get("meter_beat").unwrap_or(&4) as usize,
        meter_note: *params.get("meter_note").unwrap_or(&4) as usize,
        seq_in: *marks.get("seq_in").unwrap_or(&0),
        seq_out: *marks.get("seq_out").unwrap_or(&48000),
        loop_in: *marks.get("loop_in").unwrap_or(&0),
        loop_out: *marks.get("loop_out").unwrap_or(&0),
        sample_rate: 44_100,
        tracks: HashMap::new(),
        assets: HashMap::new(),
        regions: HashMap::new(),
        notes: vec![],

        loop_mode: false,
        tick: true,
        playhead: 0,
        zoom: 1,
        scroll_x: 0,
        scroll_y: 0,
        temp_tempo: None,
        temp_zoom: None,
        focus: (0,0),
    };

    // keep track of track index for vertical positioning
    let mut counter: u16 = 0;

    while let Some(mut track) = doc.take_child("track") {
        let t_id: &str = track.attributes.get("id").unwrap();
        let _t_id = t_id.parse::<u16>().unwrap();

        state.tracks.insert(_t_id, Track {
            id: _t_id,
            record: false,
            mute: false,
            solo: false,
            index: counter,
        });

        while let Some(region) = track.take_child("region") {

            let r_id: &str = region.attributes.get("id").unwrap();
            let a_id: &str = region.attributes.get("asset").unwrap();
            let offset: &str = region.attributes.get("offset").unwrap();
            let a_in: &str = region.attributes.get("in").unwrap();
            let a_out: &str = region.attributes.get("out").unwrap();

            state.regions.insert(r_id.parse::<u16>().unwrap(), Region {
                asset_id: a_id.parse().unwrap(),
                asset_in: a_in.parse().unwrap(),
                asset_out: a_out.parse().unwrap(),
                offset: offset.parse().unwrap(),
                track: _t_id,
            });
        }

        counter += 1;
    }

    while let Some(asset) = doc.take_child("asset") {
        let a_id: &str = asset.attributes.get("id").unwrap();
        let duration: &str = asset.attributes.get("size").unwrap();
        state.assets.insert(a_id.parse::<u16>().unwrap(), Asset {
            src: asset.attributes.get("src").unwrap().parse().unwrap(),
            duration: duration.parse().unwrap(),
            channels: 2,
            waveform: vec![],
        });
    }
    
    return state;
}