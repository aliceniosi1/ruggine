# Ruggine: Asynchronous Group Chat in Rust


**Ruggine** is a Rust project that implements a modular group chat application organized as a multi-crate workspace.

The application provides an asynchronous **TCP** client-server architecture and uses **JSON-serialized messages** for communication between components.

---

## Repository Contents

The workspace includes five main crates:

- **server** – TCP server that manages connections, users, groups, and invitations
- **client** – command-line client for interacting with the server
- **gui_client** – graphical client built with **Iced**
- **common** – shared `Message` enum and protocol definitions
- **logging** – module for periodic CPU usage logging on the server side

In addition to the source code, the repository also contains documentation and release material, including user and developer manuals.

---

## Problem Setting

Ruggine was developed as a compact but complete example of a distributed application in Rust.

The goal of the project is to support **multi-user group communication** through a clean and modular architecture, showing how to:

- design a shared messaging protocol between server and clients
- manage asynchronous communication with **Tokio**
- organize a project as a **multi-crate workspace**
- integrate both terminal and GUI frontends with the same backend

---

## Method Overview

### 1. Server

The `server` crate starts a TCP listener on port `8080` and manages:

- user registration and login
- group creation
- invitations to groups
- join and leave operations
- message broadcast to all members of a group

The server state is shared through synchronized in-memory data structures, and all networking is handled asynchronously with **Tokio**.

### 2. Shared Messaging Protocol

The `common` crate defines the `Message` enum used by both server and clients.

Supported message types include:

- `Login { username }`
- `Join { username, group }`
- `Invite { group, user }`
- `Text { group, from, content }`
- `Ack`
- `Error { reason }`
- `Create { group }`
- `Leave { group }`

This shared protocol ensures consistent serialization and deserialization of JSON messages across all components.

### 3. Command-Line Client

The `client` crate provides a terminal-based interface.

After login, users can:

- create a group with `/create <group>`
- invite another user with `/invite <group> <user>`
- join a group with `/join <group>`
- leave a group with `/leave <group>`
- send a message with `/msg <group> <message>`

If the user types plain text without a command prefix, the message is sent to the currently active group.

### 4. GUI Client

The `gui_client` crate provides a graphical interface built with **Iced**.

The GUI client supports:

- username login
- group browsing and selection
- group creation
- inviting users
- joining and leaving groups
- sending and receiving chat messages

This crate demonstrates how a Rust GUI can reuse the same backend logic and communication protocol as the CLI client.

### 5. CPU Logging Module

The `logging` crate periodically measures CPU usage and writes it to a log file.

The server runs this monitoring task asynchronously in the background, allowing system usage tracking without interrupting chat functionality.

---

## Execution Pipeline

1. Start the server with `cargo run -p server`
2. Launch one or more clients with `cargo run -p client` or `cargo run -p gui_client`
3. Log in with a username
4. Create a group or join an invited group
5. Exchange messages with other users in the same group
6. Let the logging module record CPU usage in the background

---

## Technologies Used

- **Rust 2021**
- **Tokio** for asynchronous networking
- **Serde** and **serde_json** for JSON serialization
- **Iced** for the graphical client
- **sysinfo** for CPU monitoring

---

## How to Build

Make sure Rust and Cargo are installed, then clone the repository and build the workspace:

```bash
git clone https://github.com/aliceniosi1/ruggine.git
cd ruggine
cargo build
# or release mode
cargo build --release
```

---

## How to Run

### Start the server

```bash
cargo run -p server
```

### Run the CLI client

```bash
cargo run -p client
```

### Run the GUI client

```bash
cargo run -p gui_client
```

By default, the server listens on `0.0.0.0:8080` and clients connect to `127.0.0.1:8080`.

---

## Key Contributions

- modular multi-crate Rust workspace
- asynchronous TCP group chat architecture
- shared JSON message protocol
- dual frontend support: CLI and GUI
- invitation-based group management
- background CPU usage monitoring
- additional user and developer documentation included in the repository

---

## Insights

Ruggine is not only a chat application prototype, but also a useful  project for understanding how to structure a Rust application with clear separation of responsibilities:

- **protocol layer** in `common`
- **networking and coordination layer** in `server`
- **user interaction layer** in `client` and `gui_client`
- **utility and monitoring layer** in `logging`

This makes the project a good reference for learning:

- asynchronous Rust programming
- TCP communication with Tokio
- JSON protocol design
- GUI and backend integration in Rust
- multi-crate workspace organization

## Additional Documentation

The repository also includes extra material such as:

- `Manuale_Utente.pdf`
- `Manuale_del_Progettista.pdf`
- `GroupProjectPresentation.pdf`

These documents complement the source code with both user-oriented and developer-oriented explanations.

---
