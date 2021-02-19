use ezmid::Dispatcher;

fn main() {
    let file = std::env::args().skip(1).next().unwrap_or("./youkoso.mid".to_string());
    let data = std::fs::read(&file).unwrap();
    let events = ezmid::parse(&data);

    let mut keys = vec![0; 128];
    let mut channels = vec![0; 128];

    let mut dis = Dispatcher::new(events).peekable();
    let time = std::time::Instant::now();
    while let Some(_) = dis.peek() {
        while let Some(event) = dis.peek() {
            if time.elapsed().as_secs_f64() < event.time {
                break;
            }
            match event.event.body {
                ezmid::EventBody::NoteOn { notenum, .. } => {
                    keys[notenum as usize] += 1;
                    channels[notenum as usize] = event.event.channel;
                }
                ezmid::EventBody::NoteOff { notenum, .. } => {
                    keys[notenum as usize] -= 1;
                }
                _ => {}
            }
            dis.next();
        }
        println!("{}", show_keys(&keys, &channels));
        std::thread::sleep(std::time::Duration::from_secs_f64(0.1));
    }
}

fn show_keys(keys: &[usize], channels: &[u32]) -> String {
    let mut s = String::new();
    for (key, c) in keys.iter().zip(channels) {
        s += &if *key > 0 {
            format!("\x1b[38;5;{}m|\x1b[m", c * 3 + 30)
        } else {
            format!(" ")
        };
    }
    s
}
