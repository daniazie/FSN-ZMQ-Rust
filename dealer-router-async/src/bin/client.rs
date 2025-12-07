use std::env;
use std::time::Duration;
use std::thread;

fn main() {
    let args: Vec<String> = env::args().collect();
    let client_id = args.get(1)
        .expect("Usage: program <client_id>");
    
    let context = zmq::Context::new();
    let socket = context.socket(zmq::DEALER).unwrap();
    
    socket.set_identity(client_id.as_bytes()).unwrap();
    socket.connect("tcp://localhost:5570").unwrap();
    println!("Client {} started", client_id);
    
    let mut reqs = 0;
    loop {
        reqs += 1;
        let msg = format!("request #{}", reqs);
        println!("Req #{} sent..", reqs);
        socket.send(&msg, 0).unwrap();
        
        thread::sleep(Duration::from_secs(1));
        
        if socket.poll(zmq::POLLIN, 1000).unwrap() > 0 {
            let response = socket.recv_bytes(0).unwrap();
            println!("{} received: {:?}", client_id, response);
        }
    }
}