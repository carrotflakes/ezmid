use crate::{Event, EventBody};

#[derive(Debug, Clone)]
pub struct DispatchedEvent {
    pub bpm: f64,
    pub time: f64,
    pub dtime: f64,
    pub event: Event,
}

pub struct Dispatcher {
    events: Vec<Event>,
    i: usize,
    bpm: f64,
    time: f64,
    last_beat: f64,
}

impl Dispatcher {
    pub fn new(events: Vec<Event>) -> Self {
        Self {
            events,
            i: 0,
            bpm: 120.0,
            time: 0.0,
            last_beat: 0.0,
        }
    }
}

impl Iterator for Dispatcher {
    type Item = DispatchedEvent;

    fn next(&mut self) -> Option<Self::Item> {
        if self.events.len() <= self.i {
            return None;
        }
        let mut event = self.events[self.i].clone();
        self.i += 1;
        let dtime = (event.beat - self.last_beat) * 60.0 / self.bpm;
        self.time += dtime;
        event.body = match event.body {
            EventBody::Tempo { tempo } => {
                self.bpm = tempo;
                event.body
            }
            EventBody::NoteOn {
                notenum,
                velocity,
                raw_velocity,
            } if velocity == 0.0 => EventBody::NoteOff {
                notenum,
                velocity,
                raw_velocity,
            },
            body => body,
        };
        self.last_beat = event.beat;
        Some(DispatchedEvent {
            bpm: self.bpm,
            dtime,
            time: self.time,
            event,
        })
    }
}
