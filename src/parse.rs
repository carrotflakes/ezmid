use crate::model::{Event, EventBody};
use midly::{MetaMessage, MidiMessage, Smf, TrackEventKind};

struct Channel {
    pitchbend_range: u8,
    rpn_lsb: u8,
    rpn_msb: u8,
}

impl Default for Channel {
    fn default() -> Self {
        Self {
            pitchbend_range: 2,
            rpn_lsb: 127,
            rpn_msb: 127,
        }
    }
}

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
    let mut channels: Vec<Channel> = Vec::new();
    channels.resize_with(16, Default::default);
    for (track, tick, event) in flat_events {
        match event.kind {
            TrackEventKind::Midi {
                channel: channel_no,
                message,
            } => {
                let mut channel = &mut channels[channel_no.as_int() as usize];
                match message {
                    MidiMessage::NoteOff { key, vel } => {
                        events.push(Event {
                            track: track as u8,
                            beat: tick as f64 / tpb as f64,
                            channel: channel_no.as_int(),
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
                            channel: channel_no.as_int(),
                            body: EventBody::NoteOn {
                                notenum: key.as_int(),
                                velocity: vel.as_int() as f32 / 127.0,
                                raw_velocity: vel.as_int(),
                            },
                        });
                    }
                    MidiMessage::Aftertouch { key: _, vel: _ } => {}
                    MidiMessage::Controller { controller, value } => match controller.as_int() {
                        1 => {
                            events.push(Event {
                                track: track as u8,
                                beat: tick as f64 / tpb as f64,
                                channel: channel_no.as_int(),
                                body: EventBody::Modulation {
                                    modulation: value.as_int() as f32 / 127.0,
                                    raw_modulation: value.as_int(),
                                },
                            });
                        }
                        6 => match (channel.rpn_lsb, channel.rpn_msb) {
                            (0, 0) => {
                                channel.pitchbend_range = value.as_int();
                            }
                            _ => {}
                        },
                        7 => {
                            events.push(Event {
                                track: track as u8,
                                beat: tick as f64 / tpb as f64,
                                channel: channel_no.as_int(),
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
                                channel: channel_no.as_int(),
                                body: EventBody::Pan {
                                    pan: ((value.as_int() as f32 - 64.0) / 63.0).max(-1.0),
                                    raw_pan: value.as_int(),
                                },
                            });
                        }
                        11 => {
                            events.push(Event {
                                track: track as u8,
                                beat: tick as f64 / tpb as f64,
                                channel: channel_no.as_int(),
                                body: EventBody::Expression {
                                    expression: value.as_int() as f32 / 127.0,
                                    raw_expression: value.as_int(),
                                },
                            });
                        }
                        100 => {
                            channel.rpn_lsb = value.as_int();
                        }
                        101 => {
                            channel.rpn_msb = value.as_int();
                        }
                        _ => {}
                    },
                    MidiMessage::ProgramChange { program } => {
                        events.push(Event {
                            track: track as u8,
                            beat: tick as f64 / tpb as f64,
                            channel: channel_no.as_int(),
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
                            channel: channel_no.as_int(),
                            body: EventBody::PitchBend {
                                bend: bend.as_f32() * channel.pitchbend_range as f32,
                                raw_bend: bend.as_int(),
                            },
                        });
                    }
                }
            }
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
