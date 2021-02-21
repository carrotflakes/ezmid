use crate::model::{Event, EventBody};
use midly::{MetaMessage, MidiMessage, Smf, TrackEventKind};

pub fn parse(data: &[u8]) -> Vec<Event> {
    let smf = Smf::parse(data).unwrap();
    let tpb = match smf.header.timing {
        midly::Timing::Metrical(tpb) => tpb.as_int(),
        midly::Timing::Timecode(_, _) => {
            panic!("Timecode is unsupported yet!")
        }
    };
    let mut flat_events: Vec<_> = smf
        .tracks
        .into_iter()
        .enumerate()
        .flat_map(|(track, events)| {
            let mut tick = 0;
            events.into_iter().map(move |event| {
                tick += event.delta.as_int() as u32;
                (track, tick, event)
            })
        })
        .collect();
    flat_events.sort_by_key(|e| e.1);

    let mut events = Vec::new();
    let pitchbend_range = 2;
    for (track, tick, event) in flat_events {
        match event.kind {
            TrackEventKind::Midi { channel, message } => match message {
                MidiMessage::NoteOff { key, vel } => {
                    events.push(Event {
                        track: track as u8,
                        beat: tick as f64 / tpb as f64,
                        channel: channel.as_int() as u32,
                        body: EventBody::NoteOff {
                            notenum: key.as_int(),
                            velocity: vel.as_int() as f32 / 127.0,
                            raw_velocity: vel.as_int(),
                        },
                    });
                }
                MidiMessage::NoteOn { key, vel } => {
                    events.push(Event {
                        track: track as u8,
                        beat: tick as f64 / tpb as f64,
                        channel: channel.as_int() as u32,
                        body: EventBody::NoteOn {
                            notenum: key.as_int(),
                            velocity: vel.as_int() as f32 / 127.0,
                            raw_velocity: vel.as_int(),
                        },
                    });
                }
                MidiMessage::Aftertouch { key: _, vel: _ } => {}
                MidiMessage::Controller { controller, value } => match controller.as_int() {
                    7 => {
                        events.push(Event {
                            track: track as u8,
                            beat: tick as f64 / tpb as f64,
                            channel: channel.as_int() as u32,
                            body: EventBody::Volume {
                                volume: value.as_int() as f32 / 127.0,
                                raw_volume: value.as_int(),
                            },
                        });
                    }
                    10 => {
                        events.push(Event {
                            track: track as u8,
                            beat: tick as f64 / tpb as f64,
                            channel: channel.as_int() as u32,
                            body: EventBody::Pan {
                                pan: ((value.as_int() as f32 - 64.0) / 63.0).max(-1.0),
                                raw_pan: value.as_int(),
                            },
                        });
                    }
                    _ => {}
                },
                MidiMessage::ProgramChange { program } => {
                    events.push(Event {
                        track: track as u8,
                        beat: tick as f64 / tpb as f64,
                        channel: channel.as_int() as u32,
                        body: EventBody::ProgramChange {
                            program: program.as_int(),
                        },
                    });
                }
                MidiMessage::ChannelAftertouch { vel: _ } => {}
                MidiMessage::PitchBend { bend } => {
                    events.push(Event {
                        track: track as u8,
                        beat: tick as f64 / tpb as f64,
                        channel: channel.as_int() as u32,
                        body: EventBody::PitchBend {
                            bend: bend.as_f32() * pitchbend_range as f32,
                            raw_bend: bend.as_int(),
                        },
                    });
                }
            },
            TrackEventKind::SysEx(_) => {}
            TrackEventKind::Escape(_) => {}
            TrackEventKind::Meta(m) => match m {
                MetaMessage::TrackNumber(_) => {}
                MetaMessage::Text(_) => {}
                MetaMessage::Copyright(_) => {}
                MetaMessage::TrackName(_) => {}
                MetaMessage::InstrumentName(_) => {}
                MetaMessage::Lyric(_) => {}
                MetaMessage::Marker(_) => {}
                MetaMessage::CuePoint(_) => {}
                MetaMessage::ProgramName(_) => {}
                MetaMessage::DeviceName(_) => {}
                MetaMessage::MidiChannel(_) => {}
                MetaMessage::MidiPort(_) => {}
                MetaMessage::EndOfTrack => {}
                MetaMessage::Tempo(t) => {
                    events.push(Event {
                        track: track as u8,
                        beat: tick as f64 / tpb as f64,
                        channel: 0,
                        body: EventBody::Tempo {
                            tempo: 60_000_000.0 / t.as_int() as f64,
                        },
                    });
                }
                MetaMessage::SmpteOffset(_) => {}
                MetaMessage::TimeSignature(_, _, _, _) => {}
                MetaMessage::KeySignature(_, _) => {}
                MetaMessage::SequencerSpecific(_) => {}
                MetaMessage::Unknown(_, _) => {}
            },
        }
    }
    events
}
