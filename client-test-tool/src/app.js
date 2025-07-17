import Player from './player.js';
import GameRenderer from './game-renderer.js';
import NetworkManager from './network-manager.js';
import EntityManager from './entity-manager.js';
import InputManager from './input-manager.js';

const player = new Player();
const entities = new EntityManager();
const renderer = new GameRenderer('gameCanvas');
const network = new NetworkManager(player, entities);
const input = new InputManager(player, network);

function loop(timestamp) {
    // Update logic
    player.tick(dt);
    entities.tick(dt);
    renderer.render(player, entities);
    requestAnimationFrame(loop);
}

network.on('connected', () => { /* update UI */ });
network.on('entityUpdate', (entityData) => {
    entities.update(entityData);
});

requestAnimationFrame(loop);
