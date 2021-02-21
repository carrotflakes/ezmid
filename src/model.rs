#[derive(Clone, Debug)]
pub struct Event {
    pub track: u8,
    pub beat: f64,
    pub channel: u32,
    pub body: EventBody,
}

#[derive(Clone, Debug)]
pub enum EventBody {
    NoteOn {
        notenum: u8,
        velocity: f32,
        raw_velocity: u8,
    },
    NoteOff {
        notenum: u8,
        velocity: f32,
        raw_velocity: u8,
    },
    Volume {
        volume: f32,
        raw_volume: u8,
    },
    Pan {
        pan: f32,
        raw_pan: u8,
    },
    PitchBend {
        bend: f32,
        raw_bend: i16, // in -8192..=8191
    },
    ProgramChange {
        program: u8,
    },
    Tempo {
        tempo: f64,
    },
}
