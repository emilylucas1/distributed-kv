# Distributed Key-Value Store

A distributed key-value storage system built from scratch in Rust to explore storage systems, replication, concurrency, and fault tolerance.

## Current Features

* TCP-based client/server communication
* Concurrent client handling using Rust threads
* In-memory key-value storage
* `SET`, `GET`, and `DELETE` operations
* Write-ahead logging (WAL) for persistent storage
* WAL recovery when a node restarts
* Sequence numbers for identifying operations
* Primary-to-replica replication over TCP
* Replication acknowledgments
* Independent storage logs for each node

## Architecture

The system currently uses a primary-replica architecture:

```text
                  Client
                    |
                    | TCP
                    v
            +---------------+
            |    Node 1     |
            |  Port 4000    |
            |               |
            | HashMap + WAL |
            +-------+-------+
                    |
                    | REPLICATE seq + operation
                    v
            +---------------+
            |    Node 2     |
            |  Port 4001    |
            |               |
            | HashMap + WAL |
            +---------------+
                    |
                    | OK
                    +-------→ Node 1
```

A write is assigned a sequence number by the primary node and replicated to the secondary node. The primary waits for an acknowledgment before completing the replication step.

## Running the Server

Start the primary node:

```bash
cargo run -- 4000 4001
```

Start the replica in another terminal:

```bash
cargo run -- 4001
```

The first argument specifies the node's port.

The second argument specifies the primary node's peer address. At the moment, replication is configured as a primary-to-replica relationship.

## Connecting to a Node

You can use `nc` (netcat) as a simple TCP client:

```bash
nc 127.0.0.1 4000
```

### Set a value

```text
SET name Emily
```

### Read a value

```text
GET name
```

### Delete a value

```text
DELETE name
```

Writes made through the primary are automatically replicated to the configured replica.

## Write-Ahead Log

Each node maintains its own WAL:

```text
data-4000.log
data-4001.log
```

Operations are recorded with sequence numbers:

```text
1 SET name Emily
2 SET age 19
3 DELETE age
```

When a node starts, it replays its WAL to reconstruct its in-memory state.

## Project Structure

```text
distributed-kv/
├── src/
│   ├── main.rs
│   └── storage.rs
├── Cargo.toml
├── Cargo.lock
├── .gitignore
└── README.md
```

`main.rs` handles:

* TCP connections
* client commands
* concurrent request handling
* node-to-node replication

`storage.rs` handles:

* in-memory storage
* WAL writes
* sequence numbers
* recovery
* replicated operations

## Roadmap

The project is being built incrementally toward a more fault-tolerant distributed storage system.

* [x] TCP key-value server
* [x] Concurrent client handling
* [x] Persistent WAL
* [x] WAL recovery
* [x] Operation sequence numbers
* [x] Multiple nodes
* [x] Primary-to-replica replication
* [x] Replication acknowledgments
* [ ] Replica failure handling
* [ ] Replica recovery and catch-up
* [ ] Multiple replicas
* [ ] Leader election
* [ ] Consensus with Raft
* [ ] Failure injection and testing
* [ ] Linearizability testing
* [ ] Sharding

## Goals

The long-term goal is to explore how distributed storage systems maintain consistency and recover from failures while handling concurrent requests.

The project is intentionally being implemented incrementally rather than using an existing distributed database or consensus library.
