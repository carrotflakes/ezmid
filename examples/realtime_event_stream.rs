use ezmid::Dispatcher;

fn main() {
    let file = std::env::args().skip(1).next().unwrap_or("./youkoso.mid".to_string());
    let data = std::fs::read(&file).unwrap();
    let events = ezmid::parse(&data);
    for event in Dispatcher::new(events) {
        std::thread::sleep(std::time::Duration::from_secs_f64(event.dtime));
        println!("{:?}", event);
    }
}
