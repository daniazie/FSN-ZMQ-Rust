fn main() {
    let ctx = zmq::Context::new();
    let publisher = ctx.socket(zmq::PUB).unwrap();
    assert!(publisher.bind("tcp://*:5557").is_ok());

    let collector = ctx.socket(zmq::PULL).unwrap();
    assert!(collector.bind("tcp://*:5558").is_ok());

    loop {
        let mut message = zmq::Message::new();
        collector.recv(&mut message, 0).unwrap();
        println!("server: publishing update => {}", message.as_str().unwrap());
        
        publisher.send(message, 0).unwrap();
    }
}