use rand::Rng;

fn main() {
    let ctx = zmq::Context::new();
    let subscriber = ctx.socket(zmq::SUB).unwrap();
    subscriber.set_subscribe(b"");
    assert!(subscriber.connect("tcp://localhost:5557").is_ok());

    let publisher = ctx.socket(zmq::PUSH).unwrap();
    assert!(publisher.connect("tcp://localhost:5558").is_ok());

    let mut rng = rand::rng();
    let mut message = zmq::Message::new();
    loop {
        if subscriber.poll(zmq::POLLIN, 100).unwrap() > 0 {
            subscriber.recv(&mut message, 0).unwrap();
            println!("I: received message {}", message.as_str().unwrap());
        }
        else {
            let random = rng.random_range(1..=100);
            if random < 10 {
                let msg = format!("{}", random);
                publisher.send(&msg, 0).unwrap();
                println!("I: sending message {}", random);
            }
        }

    }

}