use rand::Rng;

fn main() {
    println!("Publishing updates at weather server...");

    let context = zmq::Context::new();
    let socket = context.socket(zmq::PUB).unwrap();

    assert!(socket.bind("tcp://*:5556").is_ok());

    let mut rng = rand::rng();

    loop {
        let zipcode = rng.random_range(1..100_000);
        let temperature = rng.random_range(-80..135);
        let relhumidity = rng.random_range(10..60);

        let update = format!("{} {} {}", zipcode, temperature, relhumidity);
        socket.send(&update, 0).unwrap();
    }
}