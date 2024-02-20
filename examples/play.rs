// This example reads a MIDI file and plays it with sine waves.

use ezmid::{Dispatcher, Event, EventBody};

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("./youkoso.mid".to_string());

    // Open the MIDI file
    let data = std::fs::read(&file).unwrap();
    let events = ezmid::parse(&data);

    // Render to buffer
    let buffer: Vec<f32> = render_to_buffer(44100.0, events);
    let sec = buffer.len() as f32 / 44100.0;

    // Play the buffer
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let source = rodio::buffer::SamplesBuffer::new(1, 44100, buffer);
    stream_handle.play_raw(source).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(sec as u64 + 1));
}

pub fn render_to_buffer(sample_rate: f32, events: Vec<Event>) -> Vec<f32> {
    let scale = 1.0 / 32.0;

    let mut buffer = Vec::new();
    let mut notes: Vec<Note> = vec![];
    let mut time = 0.0;

    for event in Dispatcher::new(events) {
        for _ in 0..(event.dtime as f32 * sample_rate) as usize {
            let mut sample = 0.0;
            for note in &notes {
                sample += note.amp
                    * ((time - note.start_time) * note.frequency * std::f32::consts::TAU).sin();
            }
            buffer.push(sample * scale);
            time += 1.0 / sample_rate;
        }

        let channel = event.event.channel;
        match event.event.body {
            EventBody::NoteOn {
                notenum, velocity, ..
            } => {
                let frequency = 440.0 * 2.0f32.powf((notenum as f32 - 69.0) / 12.0);
                notes.push(Note {
                    channel,
                    note: notenum,
                    amp: (velocity as f32).powf(1.0),
                    frequency,
                    start_time: time,
                });
            }
            EventBody::NoteOff { notenum, .. } => {
                notes.retain(|n| !(n.channel == channel && n.note == notenum));
            }
            _ => {}
        }
    }

    buffer
}

struct Note {
    channel: u8,
    note: u8,
    amp: f32,
    frequency: f32,
    start_time: f32,
}
