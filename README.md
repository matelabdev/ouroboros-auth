# Ouroboros Auth 🔐

Decentralized, database-less session management built on top of the **Ouroboros L2 Mesh Network**.

## Overview

Ouroboros Auth reimagines user sessions as "photons" in flight. Instead of storing session data in a centralized database like Redis or PostgreSQL, sessions circulate within the mesh as encrypted packets. When a user logs out or a session expires, the packet is simply dropped from the ring.

### Key Features
- **Zero-Infrastructure Auth:** No database, no local state, no persistent storage needed.
- **Mesh-Wide Session Validation:** Any node in the mesh can validate a session by checking the L2 ring.
- **Instant Revocation:** Deleting a session key from the mesh instantly logs out the user across all connected services.
- **Privacy First:** Session data is encrypted using **ChaCha20-Poly1305** while in flight.

### ⚠️ Required Core Engine
This module is a plugin for the **Ouroboros L2 Mesh Network**. You **must** have a running Ouroboros node to use this auth service.
- **Get the Engine:** [matelabdev/Ouroboros](https://github.com/matelabdev/Ouroboros)

## Getting Started

### Prerequisites
- A running **Ouroboros** node on `localhost:8825` (IPC Port).

### Run the Auth Server
To start the auth service and the demo UI:
```bash
cargo run --bin auth-server
```
Then visit `http://localhost:6000` in your browser.

## API Reference

### 1. Login
`GET /api/login?u=<username>`
Creates a new session in the mesh and returns a token.

### 2. Validate
`GET /api/validate?t=<token>`
Retrieves the session data from the mesh. Returns `401` if the session is no longer in flight.

### 3. Logout
`GET /api/logout?t=<token>`
Sends a `DEL` command to the mesh to permanently destroy the session packet.

## How it Works
1. **Creation:** Upon login, a JSON payload is encrypted and injected into the mesh with the key `auth:session:<token>`.
2. **Persistence:** The data is relayed by every node in the mesh, staying "alive" as long as the ring is active.
3. **Validation:** The auth server polls the mesh via the L2 gateway to verify the token exists.
4. **Destruction:** On logout, the origin node (or any node) issues a `DEL` packet, which removes the session from the RAM of all nodes in the ring.

## License
MIT
