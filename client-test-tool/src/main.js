import Player from './player.js';
import GameRenderer from './game-renderer.js';
import { GAME_CONSTANTS } from './constants.js';

// Global instances and state
let ws = null;
let entities = new Map(); // Map<network_id, Player>
let selfPlayer = null;    // Direct reference to the local player entity
const renderer = new GameRenderer('gameCanvas');

const statusEl = document.getElementById('status');
const logEl = document.getElementById('log');

// Animation/game loop
let lastTimestamp;
function loop(timestamp) {
    if (lastTimestamp === undefined) lastTimestamp = timestamp;
    const dt = (timestamp - lastTimestamp) / 1000; // seconds

    // Tick only initialized players
    for (const player of entities.values()) {
        if (player.initialized) {
            player.tick(dt, renderer.worldBounds);
        }
    }

    renderer.render(entities);
    lastTimestamp = timestamp;
    requestAnimationFrame(loop);
}

// Logging utility
function log(message, type = 'info') {
    const entry = document.createElement('div');
    entry.className = `log-entry log-${type}`;
    entry.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
    logEl.appendChild(entry);
    logEl.scrollTop = logEl.scrollHeight;
}

// Outbound commands
function sendCommand(command) {
    if (ws && ws.readyState === WebSocket.OPEN) {
        const message = JSON.stringify(command);
        ws.send(message);
        log(`Sent: ${message}`, 'sent');
    } else {
        log('Not connected to server', 'error');
    }
}

// Connect button
document.getElementById('connect').addEventListener('click', () => {
    if (ws) ws.close();

    ws = new WebSocket(import.meta.env.VITE_WEBSOCKET_URL || 'http://ws.localhost:5000');

    ws.onopen = () => {
        statusEl.textContent = 'Connected';
        statusEl.style.color = 'green';
        log('Connected to server', 'info');
        entities.clear();
        selfPlayer = null;
    };

    ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        log(`Received: ${event.data}`, 'received');

        // Welcome message: setup identity only
        if (data.t === 'w') {
            if (data.u && data.u.length > 0) {
                const welcomeData = data.u[0].c;
                const networkId = welcomeData.network_id;
                const playerId = welcomeData.player_id;
                
                // Store identity for later use in delta/full updates
                selfPlayer = {
                    id: playerId,
                    networkId: networkId
                };

                log(`Assigned Player ID: ${playerId}`, 'info');
                log(`Assigned Network ID: ${networkId}`, 'info');
                log(`You will only receive updates for entities within ~${GAME_CONSTANTS.DEFAULT_VIEW_DISTANCE} units of your position`, 'info');

                document.getElementById('playerIdDisplay').textContent = playerId;
                document.getElementById('networkIdDisplay').textContent = networkId;
            }
        }
        // Uncomment and adapt this to support delta/full updates from your server:
        else if (data.t === 'd' || data.t === 'f') {
            data.u.forEach(update => {
                let entity = entities.get(update.i);
                const isSelf = update.i === selfPlayer?.networkId;
                
                if (!entity) {
                    entity = new Player(isSelf);
                    entities.set(update.i, entity);
                    
                    // Set up identity for self player when first seen in updates
                    if (isSelf) {
                        entity.setIds(selfPlayer.id, selfPlayer.networkId);
                        entity.self = true;
                        // Update the selfPlayer reference to the actual Player instance
                        selfPlayer = entity;
                    }
                }
                
                if (isSelf) {
                    // Client prediction reconciliation for self player
                    if (update.c.p) {
                        const serverPos = { x: update.c.p[0], y: update.c.p[1] };
                        
                        if (!entity.initialized) {
                            // First position update: set directly without correction
                            entity.setState('position', serverPos.x, serverPos.y);
                            log(`Initial position set: [${serverPos.x}, ${serverPos.y}]`, 'info');
                        } else {
                            // Subsequent updates: apply correction logic
                            const clientPos = entity.state.position;
                            
                            // Check if there's significant difference between client and server
                            const positionError = Math.abs(serverPos.x - clientPos.x) + Math.abs(serverPos.y - clientPos.y);
                            const ERROR_THRESHOLD = 2.0; // Allow small differences for smooth prediction
                            
                            if (positionError > ERROR_THRESHOLD) {
                                // Server correction: blend toward server position
                                const CORRECTION_FACTOR = 0.1; // Smooth correction over multiple frames
                                entity.state.position.x += (serverPos.x - clientPos.x) * CORRECTION_FACTOR;
                                entity.state.position.y += (serverPos.y - clientPos.y) * CORRECTION_FACTOR;
                                log(`Position correction applied: error=${positionError.toFixed(2)}`, 'debug');
                            }
                        }
                    }
                    
                    if (update.c.v) {
                        // For velocity, be more aggressive in syncing since it affects future movement
                        const serverVel = { x: update.c.v[0], y: update.c.v[1] };
                        const clientVel = entity.state.velocity;
                        
                        const velocityError = Math.abs(serverVel.x - clientVel.x) + Math.abs(serverVel.y - clientVel.y);
                        const VEL_ERROR_THRESHOLD = 5.0;
                        
                        if (velocityError > VEL_ERROR_THRESHOLD) {
                            // Correct velocity more aggressively
                            entity.setState('velocity', serverVel.x, serverVel.y);
                            log(`Velocity correction applied: error=${velocityError.toFixed(2)}`, 'debug');
                        }
                    }
                } else {
                    // Other players: apply server state directly
                    if (update.c.p) entity.setState('position', update.c.p[0], update.c.p[1]);
                    if (update.c.v) {
                        entity.setState('velocity', update.c.v[0], update.c.v[1]);
                        entity.setState('desiredVelocity', update.c.v[0], update.c.v[1]);
                    }
                }
            });
        }
    };

    ws.onclose = () => {
        statusEl.textContent = 'Disconnected';
        statusEl.style.color = 'red';
        log('Disconnected from server', 'info');
        document.getElementById('playerIdDisplay').textContent = 'Not assigned';
        document.getElementById('networkIdDisplay').textContent = 'Not assigned';

        if (selfPlayer && selfPlayer.reset) selfPlayer.reset();
        selfPlayer = null;
        entities.clear();
    };

    ws.onerror = (error) => {
        log(`WebSocket error: ${error}`, 'error');
    };
});

// Disconnect button
document.getElementById('disconnect').addEventListener('click', () => {
    if (ws) ws.close();
    if (selfPlayer && selfPlayer.stop) selfPlayer.stop();
});

// Mouse click control: move self
renderer.canvas.addEventListener('click', (e) => {
    if (!selfPlayer || !selfPlayer.state) return;
    const rect = renderer.canvas.getBoundingClientRect();
    const clickX = e.clientX - rect.left;
    const clickY = e.clientY - rect.top;

    // Convert to world coordinates
    const clickWorldPos = renderer.canvasToWorld(clickX, clickY);

    // Direction from player to click
    const dirX = clickWorldPos.x - selfPlayer.state.position.x;
    const dirY = clickWorldPos.y - selfPlayer.state.position.y;
    const dist = Math.sqrt(dirX * dirX + dirY * dirY);

    if (dist < 1e-2) return;

    // Local move
    selfPlayer.setInputDirection(dirX, dirY);

    sendCommand({ Move: { direction: [dirX, dirY] } });
});

// Stop button
document.getElementById('stop').addEventListener('click', () => {
    sendCommand("Stop");
    selfPlayer?.stop()
});

// Clear log
document.getElementById('clear').addEventListener('click', () => {
    logEl.innerHTML = '';
});

// Keyboard: spacebar to stop
document.addEventListener('keydown', (e) => {
    if (e.key === ' ') {
        sendCommand("Stop");
        if (selfPlayer && selfPlayer.stop) selfPlayer.stop();
        e.preventDefault();
    }
});

log('Test client loaded. Click Connect to start.', 'info');
requestAnimationFrame(loop);
