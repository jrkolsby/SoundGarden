use xmltree::Element;
use libcommon::{Action, Note, Key, Offset, note_list};

pub struct Store {
    thru_queue: Vec<Action>,
    intervals: Vec<Key>
}

pub fn init() -> Store {
    Store {
        thru_queue: vec![],
        intervals: vec![0,4,7], // Major Chord 1,4,7
    }
}

pub fn read(doc: &mut Element) -> Option<Store> {
    let (mut doc, notes) = note_list(doc, 0);
    let mut store: Store = init();
    store.intervals = notes.iter().map(|n| n.note).collect();
    Some(store)
}

pub fn dispatch(store: &mut Store, action: Action) {
    match action {
        Action::NoteOn(note, vol) => {
            for dnote in store.intervals.iter() {
                store.thru_queue.push(Action::NoteOn(note+dnote, vol));
            }
        },
        Action::NoteOff(note) => {
            for dnote in store.intervals.iter() {
                store.thru_queue.push(Action::NoteOff(note+dnote));
            }
        },
        _ => {}
    }
}

pub fn dispatch_requested(store: &mut Store) -> (
        Option<Vec<Action>>, // Actions for outputs
        Option<Vec<Action>>, // Actions for inputs
        Option<Vec<Action>> // Actions for client
    ) {
        let carry = store.thru_queue.clone();
        store.thru_queue.clear();
        (Some(carry), None, None)
}