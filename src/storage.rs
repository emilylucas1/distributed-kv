use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};

pub struct Storage {
    data: HashMap<String, String>,
    log: std::fs::File,
    next_seq: u64
}

impl Storage {
    pub fn new(path: &str) -> Self {
        let log = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(path)
            .unwrap();

        let mut storage = Storage {
            data: HashMap::new(),
            log,
            next_seq: 1,
        };

        storage.recover();

        storage
    }

    pub fn set(&mut self, key: String, value: String) {
        writeln!(self.log, "{} SET {} {}", self.next_seq, key, value).unwrap();
        self.log.flush().unwrap();

        self.data.insert(key, value);
        self.next_seq += 1;
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn delete(&mut self, key: &str) {
        writeln!(self.log, "{} DELETE {}", self.next_seq, key).unwrap();
        self.log.flush().unwrap();

        

        self.data.remove(key);
        self.next_seq += 1;
    }

    fn recover(&mut self) {
        let file = self.log.try_clone().unwrap();
        let reader = BufReader::new(file);

        let mut max_seq = 0;

        for line in reader.lines() {
            let line = line.unwrap();

            let parts: Vec<&str> = line.splitn(4, ' ').collect();

            match parts.as_slice() {
                [seq, "SET", key, value] => {
                    let seq: u64 = seq.parse().unwrap();
                    self.data.insert(key.to_string(), value.to_string());
                    max_seq = max_seq.max(seq);
                }

                [seq, "DELETE", key] => {
                    let seq: u64 = seq.parse().unwrap();
                    self.data.remove(*key);
                    max_seq = max_seq.max(seq);


                }

                _ => {}
            }

        }

        self.next_seq = max_seq + 1;
    }
}