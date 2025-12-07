use std::env;

fn main() {
    let context = zmq::Context::new();
    let socket = context.socket(zmq::SUB).unwrap();

    println!("Collecting updates from weather server...");
    assert!(socket.connect("tcp://localhost:5556").is_ok());

    let args: Vec<String> = env::args().collect();
    let zip_filter = if args.len() > 1 { &args[1] } else { "10001" };
    
    assert!(socket.set_subscribe(zip_filter.as_bytes()).is_ok());

    let mut total_temp = 0;

    for _ in 1..20 {
        let string = socket.recv_string(0).unwrap().unwrap();
        let inf: Vec<i64> = string.split(' ').map(|s| s.parse::<i64>().unwrap()).collect();
        let (__zipcode, temperature, _relhumidity) = (inf[0], inf[1], inf[2]);
        total_temp += temperature;

        println!("Receive temperature for zipcode {} was {} F", zip_filter, temperature);
    }

    println!("Average temperature for zipcode {} was {} F", zip_filter, total_temp/20);
}