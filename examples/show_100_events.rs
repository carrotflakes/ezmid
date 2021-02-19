fn main() {
    let file = std::env::args().skip(1).next().unwrap_or("./youkoso.mid".to_string());
    let data = std::fs::read(&file).unwrap();
    let events = ezmid::parse(&data);
    for event in &events[..100] {
        println!("{:?}", event);
    }
}
