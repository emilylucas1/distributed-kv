mod storage;


use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

use storage::Storage;

type Store = Arc<Mutex<Storage>>;
// creating one hashmap for the entire server

fn handle_replication(parts: &[&str], store: &Store) -> String {
    match parts {
        ["REPLICATE", seq, "SET", key, value] => {
            let seq: u64 = match seq.parse() {
                Ok(seq) => seq,
                Err(_) => return "ERROR\n".to_string(),
            };

            println!(
                "Received replicated SET: seq={}, key={}, value={}",
                seq, key, value
            );

            let mut store = store.lock().unwrap();

            store.apply_replicated_set(
                seq,
                key.to_string(),
                value.to_string(),
            );

            "OK\n".to_string()
        }

        ["REPLICATE", seq, "DELETE", key] => {
            let seq: u64 = match seq.parse() {
                Ok(seq) => seq,
                Err(_) => return "ERROR\n".to_string(),
            };

            println!(
                "Received replicated DELETE: seq={}, key={}",
                seq, key
            );

            let mut store = store.lock().unwrap();

            store.apply_replicated_delete(seq, key);

            "OK\n".to_string()
        }

        _ => "ERROR\n".to_string(),
    }
}

fn send_to_node(address: &str, message: &str) -> std::io::Result<()> {
    let mut stream = TcpStream::connect(address)?;
    stream.write_all(message.as_bytes())?;

    Ok(())
}

fn handle_client(mut stream: TcpStream, store: Store, peer_address: Option<String>,) {
    let reader_stream = stream.try_clone().unwrap();
    let mut reader = BufReader::new(reader_stream);

    loop {
        let mut request = String::new();

        match reader.read_line(&mut request) {
            Ok(0) => break, // client disconnected
            Ok(_) => {}
            Err(_) => break,
        }

        let parts: Vec<&str> = request.trim().split(' ').collect();

        let response = match parts.as_slice() {
            ["SET", key, value] => {
                let seq = {
                    let mut store = store.lock().unwrap();
                    store.set(key.to_string(), value.to_string())
                };
            
                if let Some(peer) = &peer_address {
                    let message = format!(
                        "REPLICATE {} SET {} {}\n",
                        seq, key, value
                    );
            
                    if let Err(error) = send_to_node(peer, &message) {
                        eprintln!("Replication failed: {}", error);
                        return;
                    }
                }
            
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
                let seq = {
                    let mut store = store.lock().unwrap();
                    store.delete(key)
                };
            
                if let Some(peer) = &peer_address {
                    let message = format!(
                        "REPLICATE {} DELETE {}\n",
                        seq, key
                    );
            
                    if let Err(error) = send_to_node(peer, &message) {
                        eprintln!("Replication failed: {}", error);
                        return;
                    }
                }
            
                "OK\n".to_string()
            }

            ["REPLICATE", ..] => {
                handle_replication(&parts, &store)
            }
            _ => "ERROR\n".to_string(),
        };

        stream.write_all(response.as_bytes()).unwrap();
    }
}


fn main() {
    let args: Vec<String> = std::env::args().collect();

    let peer_address = if args.len() > 2 {
        Some(format!("127.0.0.1:{}", args[2]))
    } else {
        None
    };

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

        let peer_address = peer_address.clone();

        std::thread::spawn(move || {
            handle_client(stream, store, peer_address);
        });
    }
}