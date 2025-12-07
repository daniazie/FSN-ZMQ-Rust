use std::thread;
use std::env;

fn server_worker(context: zmq::Context, id: usize) {
    let worker = context.socket(zmq::DEALER).unwrap();
    worker.connect("inproc://backend").unwrap();
    println!("Worker#{} started", id);
    
    loop {
        let ident = worker.recv_bytes(0).unwrap();
        let msg = worker.recv_bytes(0).unwrap();
        
        println!("Worker#{} received {:?} from {:?}", id, msg, ident);
        
        worker.send(&ident, zmq::SNDMORE).unwrap();
        worker.send(&msg, 0).unwrap();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let num_workers: usize = args.get(1)
        .expect("Usage: program <num_workers>")
        .parse()
        .expect("num_workers must be a number");
    
    let context = zmq::Context::new();
    
    // Frontend socket (faces clients)
    let frontend = context.socket(zmq::ROUTER).unwrap();
    frontend.bind("tcp://*:5570").unwrap();
    
    // Backend socket (faces workers)
    let backend = context.socket(zmq::DEALER).unwrap();
    backend.bind("inproc://backend").unwrap();
    
    // Spawn worker threads
    let mut handles = vec![];
    for i in 0..num_workers {
        let ctx = context.clone();
        let handle = thread::spawn(move || {
            server_worker(ctx, i);
        });
        handles.push(handle);
    }
    
    // Run proxy (blocks forever)
    zmq::proxy(&frontend, &backend).unwrap();
}