fn main() {
    let context = zmq::Context::new();
    
    println!("Connecting to hello world server...");
    let socket = context.socket(zmq::REQ).unwrap();

    assert!(socket.connect("tcp://localhost:5555").is_ok());

    let mut message = zmq::Message::new();

    for request in 1..10 {
        println!("Sending request {}!", request);
        socket.send("Hello", 0).unwrap();

        socket.recv(&mut message, 0).unwrap();
        println!("Received reply {} [ {} ]", request, message.as_str().unwrap());
    }
}