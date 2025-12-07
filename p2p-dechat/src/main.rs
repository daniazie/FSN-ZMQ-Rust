use std::io;
use std::net::{TcpStream, ToSocketAddrs};
use std::thread;
use std::time::Duration;
use local_ip_address::local_ip;

fn search_nameserver(ip_mask: &str, port_nameserver: u16) -> Option<String> {
    let context = zmq::Context::new();
    let socket = context.socket(zmq::SUB).unwrap();
    
    // Connect to all possible IPs in the subnet
    for last in 1..255 {
        let target_ip = format!("tcp://{}.{}:{}", ip_mask, last, port_nameserver);
        socket.connect(&target_ip).ok();
    }
    
    socket.set_rcvtimeo(2000).unwrap(); // 2 second timeout
    socket.set_subscribe(b"NAMESERVER").unwrap();
    
    match socket.recv_string(0) {
        Ok(Ok(res)) => {
            let parts: Vec<&str> = res.split(':').collect();
            if parts.len() >= 2 && parts[0] == "NAMESERVER" {
                Some(parts[1].to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

fn beacon_nameserver(local_ip_addr: String, port_nameserver: u16) {
    let context = zmq::Context::new();
    let socket = context.socket(zmq::PUB).unwrap();
    let bind_addr = format!("tcp://{}:{}", local_ip_addr, port_nameserver);
    socket.bind(&bind_addr).unwrap();
    println!("local p2p name server bind to {}.", bind_addr);
    
    loop {
        thread::sleep(Duration::from_secs(1));
        let msg = format!("NAMESERVER:{}", local_ip_addr);
        if socket.send(&msg, 0).is_err() {
            break;
        }
    }
}

fn user_manager_nameserver(local_ip_addr: String, port_subscribe: u16) {
    let context = zmq::Context::new();
    let socket = context.socket(zmq::REP).unwrap();
    let bind_addr = format!("tcp://{}:{}", local_ip_addr, port_subscribe);
    socket.bind(&bind_addr).unwrap();
    println!("local p2p db server activated at {}.", bind_addr);
    
    loop {
        match socket.recv_string(0) {
            Ok(Ok(user_req)) => {
                let parts: Vec<&str> = user_req.split(':').collect();
                if parts.len() >= 2 {
                    println!("user registration '{}' from '{}'.", parts[1], parts[0]);
                }
                socket.send("ok", 0).ok();
            }
            _ => break,
        }
    }
}

fn relay_server_nameserver(local_ip_addr: String, port_chat_publisher: u16, port_chat_collector: u16) {
    let context = zmq::Context::new();
    let publisher = context.socket(zmq::PUB).unwrap();
    let collector = context.socket(zmq::PULL).unwrap();
    
    let pub_addr = format!("tcp://{}:{}", local_ip_addr, port_chat_publisher);
    let col_addr = format!("tcp://{}:{}", local_ip_addr, port_chat_collector);
    
    publisher.bind(&pub_addr).unwrap();
    collector.bind(&col_addr).unwrap();
    println!("local p2p relay server activated at {} & {}.", pub_addr, col_addr);
    
    loop {
        match collector.recv_string(0) {
            Ok(Ok(message)) => {
                println!("p2p-relay:<==>{}", message);
                let relay_msg = format!("RELAY:{}", message);
                publisher.send(&relay_msg, 0).ok();
            }
            _ => break,
        }
    }
}

fn get_local_ip() -> String {
    match local_ip() {
        Ok(ip) => ip.to_string(),
        Err(_) => "127.0.0.1".to_string(),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        println!("usage is 'cargo run -- _user-name_'.");
        return;
    }
    
    println!("starting p2p chatting program.");
    
    let port_nameserver: u16 = 9001;
    let port_chat_publisher: u16 = 9002;
    let port_chat_collector: u16 = 9003;
    let port_subscribe: u16 = 9004;
    
    let user_name = &args[1];
    let ip_addr = get_local_ip();
    let ip_mask: String = ip_addr.rsplitn(2, '.').nth(1).unwrap_or("192.168.1").to_string();
    
    println!("searching for p2p server.");
    
    let name_server_ip_addr = search_nameserver(&ip_mask, port_nameserver);
    let ip_addr_p2p_server = if let Some(server_ip) = name_server_ip_addr {
        println!("p2p server found at {}, and p2p client mode is activated.", server_ip);
        server_ip
    } else {
        println!("p2p server is not found, and p2p server mode is activated.");
        
        let ip_beacon = ip_addr.clone();
        thread::spawn(move || beacon_nameserver(ip_beacon, port_nameserver));
        println!("p2p beacon server is activated.");
        
        let ip_db = ip_addr.clone();
        thread::spawn(move || user_manager_nameserver(ip_db, port_subscribe));
        println!("p2p subscriber database server is activated.");
        
        let ip_relay = ip_addr.clone();
        thread::spawn(move || relay_server_nameserver(ip_relay, port_chat_publisher, port_chat_collector));
        println!("p2p message relay server is activated.");
        
        ip_addr.clone()
    };
    
    println!("starting user registration procedure.");
    
    let context = zmq::Context::new();
    let db_client_socket = context.socket(zmq::REQ).unwrap();
    let db_addr = format!("tcp://{}:{}", ip_addr_p2p_server, port_subscribe);
    db_client_socket.connect(&db_addr).unwrap();
    
    let reg_msg = format!("{}:{}", ip_addr, user_name);
    db_client_socket.send(&reg_msg, 0).unwrap();
    
    match db_client_socket.recv_string(0) {
        Ok(Ok(response)) if response == "ok" => {
            println!("user registration to p2p server completed.");
        }
        _ => {
            println!("user registration to p2p server failed.");
        }
    }
    
    println!("starting message transfer procedure.");
    
    let relay_context = zmq::Context::new();
    let p2p_rx = relay_context.socket(zmq::SUB).unwrap();
    p2p_rx.set_subscribe(b"RELAY").unwrap();
    let rx_addr = format!("tcp://{}:{}", ip_addr_p2p_server, port_chat_publisher);
    p2p_rx.connect(&rx_addr).unwrap();
    
    let p2p_tx = relay_context.socket(zmq::PUSH).unwrap();
    let tx_addr = format!("tcp://{}:{}", ip_addr_p2p_server, port_chat_collector);
    p2p_tx.connect(&tx_addr).unwrap();
    
    println!("starting autonomous message transmit and receive scenario.");
    
    use rand::Rng;
    let mut rng = rand::rng();
    
    loop {
        match p2p_rx.poll(zmq::POLLIN, 100) {
            Ok(events) if events != 0 => {
                if let Ok(Ok(message)) = p2p_rx.recv_string(0) {
                    let parts: Vec<&str> = message.split(':').collect();
                    if parts.len() >= 3 {
                        println!("p2p-recv::<<== {}:{}", parts[1], parts[2]);
                    }
                }
            }
            Ok(_) | Err(_) => {
                let rand: i32 = rng.random_range(1..=100);
                if rand < 10 {
                    thread::sleep(Duration::from_secs(3));
                    let msg = format!("({},{}:ON)", user_name, ip_addr);
                    p2p_tx.send(&msg, 0).ok();
                    println!("p2p-send::==>>{}", msg);
                } else if rand > 90 {
                    thread::sleep(Duration::from_secs(3));
                    let msg = format!("({},{}:OFF)", user_name, ip_addr);
                    p2p_tx.send(&msg, 0).ok();
                    println!("p2p-send::==>>{}", msg);
                }
            }
        }
    }
}