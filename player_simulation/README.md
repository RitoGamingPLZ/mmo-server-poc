# MMO Player Simulation

A Node.js script to simulate N players connecting to the MMO server and sending random movement commands.

## Installation

```bash
npm install
```

## Usage

### Basic Usage
```bash
node player_simulation.js
```

### With Options
```bash
# Simulate 20 players with 500ms movement interval
node player_simulation.js -n 20 -i 500

# Connect to different server
node player_simulation.js -s ws://localhost:8080 -n 10
```

### Available Options

- `-n, --players <count>`: Number of players to simulate (default: 5)
- `-i, --interval <ms>`: Movement interval in milliseconds (default: 1000)
- `-s, --server <url>`: WebSocket server URL (default: ws://localhost:5000)
- `-h, --help`: Show help message

## Features

- Simulates N WebSocket client connections
- Sends random movement commands at configurable intervals
- Sends heartbeat messages every 10 seconds
- Tracks connection statistics and latency
- Graceful shutdown with Ctrl+C

## Output

The script displays:
- Connection status for each player
- Real-time statistics every 5 seconds
- Total messages sent/received
- Average latency
- Uptime and message rate

## Example Output

```
[10:30:15] Starting simulation with 5 players
[10:30:15] Server: ws://localhost:5000
[10:30:15] Move interval: 1000ms
[10:30:15] Player 1 connected
[10:30:15] Player 2 connected
...

=== SIMULATION STATS ===
Connected Players: 5/5
Messages Sent: 142
Messages Received: 89
Average Latency: 12ms
Uptime: 30s
Messages/sec: 4
========================
```