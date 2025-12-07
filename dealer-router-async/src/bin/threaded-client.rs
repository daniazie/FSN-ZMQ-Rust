use std::env;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;
use std::thread;

fn recv_handler(socket: Arc<zmq::Socket>, identity: String) {
    loop {
        // Poll with 1000ms timeout
        if socket.poll(zmq::POLLIN, 1000).unwrap() > 0 {
            let msg = socket.recv_bytes(0).unwrap();
            println!("{} received: {:?}", identity, msg);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let client_id = args.get(1)
        .expect("Usage: program <client_id>")
        .clone();
    
    let context = zmq::Context::new();
    let socket = context.socket(zmq::DEALER).unwrap();
    
    socket.set_identity(client_id.as_bytes()).unwrap();
    socket.connect("tcp://localhost:5570").unwrap();
    println!("Client {} started", client_id);
    
    // Channel to send messages from sender thread to main thread
    let (tx, rx) = mpsc::channel();
    
    // Spawn sender thread
    let sender_id = client_id.clone();
    thread::spawn(move || {
        let mut reqs = 0;
        loop {
            reqs += 1;
            let msg = format!("request #{}", reqs);
            println!("Req #{} sent..", reqs);
            tx.send(msg).unwrap();
            thread::sleep(Duration::from_secs(1));
        }
    });
    
    // Main thread handles socket I/O
    loop {
        // Try to receive from channel (non-blocking)
        if let Ok(msg) = rx.try_recv() {
            socket.send(&msg, 0).unwrap();
        }
        
        // Poll for responses
        if socket.poll(zmq::POLLIN, 100).unwrap() > 0 {
            let response = socket.recv_bytes(0).unwrap();
            println!("{} received: {:?}", client_id, response);
        }
    }
}