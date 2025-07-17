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

    // Tick only the self player for client prediction
    for (const player of entities.values()) player.tick(dt, renderer.worldBounds);

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

    ws = new WebSocket('ws://localhost:5000');

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

        // Welcome message: initialize local player
        if (data.t === 'w') {
            if (data.u && data.u.length > 0) {
                const welcomeData = data.u[0].c;
                const networkId = welcomeData.network_id;
                let player = entities.get(networkId);
                if (!player) {
                    player = new Player(true);
                    entities.set(networkId, player);
                }
                player.setIds(welcomeData.player_id, networkId);
                player.self = true;
                player.setState("position", 500, 500); // or use server value
                selfPlayer = player;

                log(`Assigned Player ID: ${player.id}`, 'info');
                log(`Assigned Network ID: ${player.networkId}`, 'info');
                log(`You will only receive updates for entities within ~${GAME_CONSTANTS.DEFAULT_VIEW_DISTANCE} units of your position`, 'info');

                document.getElementById('playerIdDisplay').textContent = player.id;
                document.getElementById('networkIdDisplay').textContent = player.networkId;
            }
        }
        // Uncomment and adapt this to support delta/full updates from your server:
        else if (data.t === 'd' || data.t === 'f') {
            data.u.forEach(update => {
                let entity = entities.get(update.i);
                if (!entity) {
                    entity = new Player(update.i === selfPlayer?.networkId);
                    entities.set(update.i, entity);
                }
                if (update.c.p) entity.setState('position', update.c.p[0], update.c.p[1]);
                if (update.c.v) {
                    entity.setState('velocity', update.c.v[0], update.c.v[1]);
                    entity.setState('desiredVelocity', update.c.v[0], update.c.v[1]);
                }
                if (entity.networkId === selfPlayer?.networkId) entity.self = true;
            });
        }
    };

    ws.onclose = () => {
        statusEl.textContent = 'Disconnected';
        statusEl.style.color = 'red';
        log('Disconnected from server', 'info');
        document.getElementById('playerIdDisplay').textContent = 'Not assigned';
        document.getElementById('networkIdDisplay').textContent = 'Not assigned';

        if (selfPlayer) selfPlayer.reset();
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
    if (selfPlayer) selfPlayer.stop();
});

// Mouse click control: move self
renderer.canvas.addEventListener('click', (e) => {
    if (!selfPlayer) return;
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
        e.preventDefault();
    }
});

log('Test client loaded. Click Connect to start.', 'info');
requestAnimationFrame(loop);
