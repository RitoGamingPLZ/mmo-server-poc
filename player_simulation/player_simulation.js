#!/usr/bin/env node

const WebSocket = require('ws');
const { performance } = require('perf_hooks');

class PlayerSimulation {
    constructor() {
        this.players = [];
        this.isRunning = false;
        this.serverUrl = 'ws://localhost:5000';
        this.moveInterval = 1000; // ms
        this.totalMessages = 0;
        this.connectedCount = 0;
        this.startTime = 0;
        this.moveTimer = null;
        this.heartbeatTimer = null;
        this.statsTimer = null;
        this.latencySum = 0;
        this.latencyCount = 0;
        this.totalBytesSent = 0;
        this.totalBytesReceived = 0;
    }

    log(message) {
        const timestamp = new Date().toLocaleTimeString();
        console.log(`[${timestamp}] ${message}`);
    }

    createPlayer(playerId) {
        const player = {
            id: playerId,
            socket: null,
            connected: false,
            position: { x: 500, y: 500 }, // Start in center
            lastPingTime: 0,
            latency: 0,
            messagesSent: 0,
            messagesReceived: 0,
            bytesSent: 0,
            bytesReceived: 0,
            lastMessageSize: 0
        };

        try {
            player.socket = new WebSocket(this.serverUrl);
            
            player.socket.on('open', () => {
                player.connected = true;
                this.connectedCount++;
                this.log(`Player ${playerId} connected`);
            });

            player.socket.on('message', (data) => {
                player.messagesReceived++;
                const messageSize = Buffer.byteLength(data);
                player.bytesReceived += messageSize;
                player.lastMessageSize = messageSize;
                this.totalBytesReceived += messageSize;
                
                try {
                    const message = JSON.parse(data.toString());
                    
                    // Calculate latency if this is a response to our ping
                    if (player.lastPingTime > 0) {
                        player.latency = performance.now() - player.lastPingTime;
                        this.latencySum += player.latency;
                        this.latencyCount++;
                        player.lastPingTime = 0;
                    }

                    // Handle different message types
                    if (message.t === 'f') { // full_sync
                        // Full sync received
                        if (message.u && message.u.length > 0) {
                            // Update player position from first entity (assuming it's this player)
                            const firstEntity = message.u[0];
                            if (firstEntity.c && firstEntity.c.p) {
                                player.position.x = firstEntity.c.p[0];
                                player.position.y = firstEntity.c.p[1];
                            }
                        }
                    } else if (message.t === 'd') { // delta_update
                        // Delta update received
                        if (message.u) {
                            message.u.forEach(entityUpdate => {
                                if (entityUpdate.c && entityUpdate.c.p) {
                                    player.position.x = entityUpdate.c.p[0];
                                    player.position.y = entityUpdate.c.p[1];
                                }
                            });
                        }
                    } else if (message.t === 'w') { // welcome
                        // Welcome message received
                    } else if (message.t === 'entity_removed') {
                        // Another player disconnected
                    }
                } catch (e) {
                    // Handle non-JSON messages (like heartbeat responses)
                }
            });

            player.socket.on('close', () => {
                player.connected = false;
                this.connectedCount--;
                this.log(`Player ${playerId} disconnected`);
            });

            player.socket.on('error', (error) => {
                this.log(`Player ${playerId} error: ${error.message}`);
            });

        } catch (error) {
            this.log(`Failed to create player ${playerId}: ${error.message}`);
        }

        return player;
    }

    sendRandomMove(player) {
        if (!player.connected || !player.socket) return;

        let command, messageStr;
        
        // 30% chance to stop, 70% chance to move
        if (Math.random() < 0.7) {
            // Send stop command
            command = "Stop";
            messageStr = JSON.stringify(command);
        } else {
            // Generate random direction vector
            const angle = Math.random() * 2 * Math.PI;
            const direction = [Math.cos(angle), Math.sin(angle)];
            
            command = {
                Move: {
                    direction: direction
                }
            };
            messageStr = JSON.stringify(command);
        }

        try {
            const messageSize = Buffer.byteLength(messageStr);
            player.socket.send(messageStr);
            player.messagesSent++;
            player.bytesSent += messageSize;
            this.totalMessages++;
            this.totalBytesSent += messageSize;
            player.lastPingTime = performance.now();
        } catch (error) {
            this.log(`Failed to send command for player ${player.id}: ${error.message}`);
        }
    }

    printStats() {
        const uptime = Math.round((performance.now() - this.startTime) / 1000);
        const avgLatency = this.latencyCount > 0 ? Math.round(this.latencySum / this.latencyCount) : 0;
        const totalMessagesReceived = this.players.reduce((sum, p) => sum + p.messagesReceived, 0);
        const totalBytesSentByPlayers = this.players.reduce((sum, p) => sum + p.bytesSent, 0);
        const totalBytesReceivedByPlayers = this.players.reduce((sum, p) => sum + p.bytesReceived, 0);
        
        const avgMessageSizeSent = this.totalMessages > 0 ? Math.round(this.totalBytesSent / this.totalMessages) : 0;
        const avgMessageSizeReceived = totalMessagesReceived > 0 ? Math.round(this.totalBytesReceived / totalMessagesReceived) : 0;
        
        console.log('\n=== SIMULATION STATS ===');
        console.log(`Connected Players: ${this.connectedCount}/${this.players.length}`);
        console.log(`Messages Sent: ${this.totalMessages}`);
        console.log(`Messages Received: ${totalMessagesReceived}`);
        console.log(`Bytes Sent: ${this.totalBytesSent} (${Math.round(this.totalBytesSent / 1024)}KB)`);
        console.log(`Bytes Received: ${this.totalBytesReceived} (${Math.round(this.totalBytesReceived / 1024)}KB)`);
        console.log(`Avg Message Size Sent: ${avgMessageSizeSent} bytes`);
        console.log(`Avg Message Size Received: ${avgMessageSizeReceived} bytes`);
        console.log(`Average Latency: ${avgLatency}ms`);
        console.log(`Uptime: ${uptime}s`);
        console.log(`Messages/sec: ${Math.round(this.totalMessages / uptime)}`);
        console.log(`Bandwidth In: ${Math.round(this.totalBytesReceived / uptime)} bytes/sec`);
        console.log(`Bandwidth Out: ${Math.round(this.totalBytesSent / uptime)} bytes/sec`);
        console.log('========================\n');
    }

    async start(playerCount = 5, moveIntervalMs = 1000, serverUrl = 'ws://localhost:5000') {
        if (this.isRunning) {
            this.log('Simulation already running!');
            return;
        }

        this.serverUrl = serverUrl;
        this.moveInterval = moveIntervalMs;
        this.isRunning = true;
        this.startTime = performance.now();
        this.totalMessages = 0;
        this.connectedCount = 0;
        this.latencySum = 0;
        this.latencyCount = 0;
        this.totalBytesSent = 0;
        this.totalBytesReceived = 0;

        this.log(`Starting simulation with ${playerCount} players`);
        this.log(`Server: ${this.serverUrl}`);
        this.log(`Move interval: ${this.moveInterval}ms`);

        // Create players with staggered connections (100ms apart)
        this.players = [];
        for (let i = 1; i <= playerCount; i++) {
            setTimeout(() => {
                this.players.push(this.createPlayer(i));
            }, i * 100);
        }

        // Wait for connections to establish
        await new Promise(resolve => setTimeout(resolve, 2000));

        // Start movement timer
        this.moveTimer = setInterval(() => {
            this.players.forEach(player => {
                if (player.connected) {
                    this.sendRandomMove(player);
                }
            });
        }, this.moveInterval);

        // Start stats timer (every 5 seconds)
        this.statsTimer = setInterval(() => {
            this.printStats();
        }, 5000);

        this.log('Simulation started!');
    }

    stop() {
        if (!this.isRunning) {
            this.log('Simulation not running!');
            return;
        }

        this.isRunning = false;

        // Clear timers
        if (this.moveTimer) {
            clearInterval(this.moveTimer);
            this.moveTimer = null;
        }
        if (this.heartbeatTimer) {
            clearInterval(this.heartbeatTimer);
            this.heartbeatTimer = null;
        }
        if (this.statsTimer) {
            clearInterval(this.statsTimer);
            this.statsTimer = null;
        }

        // Disconnect all players
        this.players.forEach(player => {
            if (player.socket) {
                player.socket.close();
            }
        });

        this.players = [];
        this.connectedCount = 0;
        this.log('Simulation stopped!');
    }
}

// CLI interface
function printUsage() {
    console.log(`
Usage: node player_simulation.js [options]

Options:
  -n, --players <count>     Number of players to simulate (default: 5)
  -i, --interval <ms>       Movement interval in milliseconds (default: 1000)
  -s, --server <url>        WebSocket server URL (default: ws://localhost:5000)
  -h, --help               Show this help message

Examples:
  node player_simulation.js
  node player_simulation.js -n 10 -i 500
  node player_simulation.js --players 20 --interval 2000 --server ws://localhost:5000
`);
}

// Parse command line arguments
function parseArgs() {
    const args = process.argv.slice(2);
    const options = {
        players: 5,
        interval: 1000,
        server: 'ws://localhost:5000'
    };

    for (let i = 0; i < args.length; i++) {
        const arg = args[i];
        
        if (arg === '-h' || arg === '--help') {
            printUsage();
            process.exit(0);
        } else if (arg === '-n' || arg === '--players') {
            options.players = parseInt(args[++i]) || 5;
        } else if (arg === '-i' || arg === '--interval') {
            options.interval = parseInt(args[++i]) || 1000;
        } else if (arg === '-s' || arg === '--server') {
            options.server = args[++i] || 'ws://localhost:5000';
        }
    }

    return options;
}

// Main execution
if (require.main === module) {
    const options = parseArgs();
    const simulation = new PlayerSimulation();

    // Handle graceful shutdown
    process.on('SIGINT', () => {
        console.log('\nReceived SIGINT, stopping simulation...');
        simulation.stop();
        process.exit(0);
    });

    process.on('SIGTERM', () => {
        console.log('\nReceived SIGTERM, stopping simulation...');
        simulation.stop();
        process.exit(0);
    });

    console.log('🚀 MMO Player Simulation Starting...');
    console.log(`Players: ${options.players}`);
    console.log(`Move Interval: ${options.interval}ms`);
    console.log(`Server: ${options.server}`);
    console.log('Press Ctrl+C to stop simulation\n');

    simulation.start(options.players, options.interval, options.server);
}

module.exports = PlayerSimulation;