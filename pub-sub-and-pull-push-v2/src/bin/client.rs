use rand::Rng;
use std::env;

fn main() {
    let argv: Vec<String> = env::args().collect();
    let ctx = zmq::Context::new();
    let subscriber = ctx.socket(zmq::SUB).unwrap();
    subscriber.set_subscribe(b"");
    assert!(subscriber.connect("tcp://localhost:5557").is_ok());

    let publisher = ctx.socket(zmq::PUSH).unwrap();
    assert!(publisher.connect("tcp://localhost:5558").is_ok());

    let clientID = &argv[1];
    let mut rng = rand::rng();

    let mut message = zmq::Message::new();
    loop {
        if subscriber.poll(zmq::POLLIN, 100).unwrap() > 0 {
            subscriber.recv(&mut message, 0).unwrap();
            println!("{}: receive status => {}", clientID, message.as_str().unwrap());
        }
        else {
            let random = rng.random_range(1..=100);
            if random < 10 {
                let msg = format!("( {} :ON)", clientID);
                publisher.send(&msg, 0).unwrap();
                println!("{}: send status - activated", clientID);
            }
            else if random > 90 {
                let msg = format!("( {} :OFF)", clientID);
                publisher.send_str(&msg, 0).unwrap();
                println!("{}: send status - deactivated", clientID)
            }
        

        }
    }
}