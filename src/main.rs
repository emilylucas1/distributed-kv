mod storage;


use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use storage::Storage;

type Store = Arc<Mutex<Storage>>;
// creating one hashmap for the entire server

fn handle_client(mut stream: TcpStream, store: Store) {
    let reader_stream = stream.try_clone().unwrap();
    let mut reader = BufReader::new(reader_stream);

    loop {
        let mut request = String::new();

        match reader.read_line(&mut request) {
            Ok(0) => break, // client disconnected
            Ok(_) => {}
            Err(_) => break,
        }

        let parts: Vec<&str> = request.trim().splitn(3, ' ').collect();

        let response = match parts.as_slice() {
            ["SET", key, value] => {
                let mut store = store.lock().unwrap();
                store.set(key.to_string(), value.to_string());
                "OK\n".to_string()
            }

            ["GET", key] => {
                let store = store.lock().unwrap();

                match store.get(key) {
                    Some(value) => format!("{}\n", value),
                    None => "NOT_FOUND\n".to_string(),
                }
            }

            ["DELETE", key] => {
                let mut store = store.lock().unwrap();
                store.delete(key);
                "OK\n".to_string()
            }

            _ => "ERROR\n".to_string(),
        };

        stream.write_all(response.as_bytes()).unwrap();
    }
}
fn main() {
    let args: Vec<String> = std::env::args().collect();

    let port = if args.len() > 1 {
        &args[1]
    } else {
        "4000"
    };

    let address = format!("127.0.0.1:{}", port);

    let listener = TcpListener::bind(&address).unwrap();

    let log_path = format!("data-{}.log", port);

    let store: Store = Arc::new(Mutex::new(Storage::new(&log_path)));

    println!("Server listening on {}", address);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let store = Arc::clone(&store);

        std::thread::spawn(move || {
            handle_client(stream, store);
        });
    }
}