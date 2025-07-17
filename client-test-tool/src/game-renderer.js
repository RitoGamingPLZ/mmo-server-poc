import { GAME_CONSTANTS } from './constants.js';

class GameRenderer {
    constructor(canvasId) {
        this.canvas = document.getElementById(canvasId);
        this.ctx = this.canvas.getContext('2d');
        this.viewDistance = GAME_CONSTANTS.DEFAULT_VIEW_DISTANCE;
        this.scale = GAME_CONSTANTS.DEFAULT_SCALE;
        this.worldBounds = GAME_CONSTANTS.DEFAULT_WORLD_BOUNDS;
        this.resizeCanvas();
        window.addEventListener('resize', () => this.resizeCanvas());
    }
    
    get width() { return this.canvas.width; }
    get height() { return this.canvas.height; }

     setWorldBounds(bounds) {
        this.worldBounds = bounds;
        this.resizeCanvas(); // recalc scale
    }
    
    resizeCanvas() {
        const gameArea = document.querySelector('.game-area');
        const availableWidth = gameArea.clientWidth;
        const availableHeight = gameArea.clientHeight;
        
        // Make canvas square and not exceed max size
        const maxSize = Math.min(availableWidth, availableHeight, GAME_CONSTANTS.MAX_CANVAS_SIZE);
        this.canvas.width = maxSize;
        this.canvas.height = maxSize;

        // Compute scale to fit world in canvas, keeping aspect ratio
        const scaleX = this.canvas.width / this.worldBounds.x;
        const scaleY = this.canvas.height / this.worldBounds.y;
        this.scale = Math.min(scaleX, scaleY);
        this.render();
    }
    
    worldToCanvas(worldX, worldY) {
        return { x: worldX * this.scale, y: worldY * this.scale };
    }
    canvasToWorld(canvasX, canvasY) {
        return { x: canvasX / this.scale, y: canvasY / this.scale };
    }

    
    drawPlayer(player, {
        viewDistance = null,
        scale = 1,
        canvasWidth = null,
        canvasHeight = null
    } = {}) {
        const pos = player.state.position;
        const velocity = player.state.velocity;
        const self = player.self;

        const canvasPos = this.worldToCanvas(pos.x, pos.y);


        // World-to-canvas conversion if needed
        // const canvasPos = this.worldToCanvas(pos.x, pos.y);

        // --- Only for self: draw view radius and gray mask ---
        if (self && viewDistance && canvasWidth && canvasHeight) {
            const radius = viewDistance * scale;
            this.ctx.beginPath();
            this.ctx.arc(canvasPos.x, canvasPos.y, radius * scale, 0, 2 * Math.PI);
            this.ctx.strokeStyle = GAME_CONSTANTS.COLORS.VIEW_RADIUS;
            this.ctx.lineWidth = 2;
            this.ctx.stroke();

            this.ctx.save();
            this.ctx.fillStyle = GAME_CONSTANTS.COLORS.GRAY_MASK;
            this.ctx.fillRect(0, 0, canvasWidth, canvasHeight);

            this.ctx.globalCompositeOperation = 'destination-out';
            this.ctx.beginPath();
            this.ctx.arc(canvasPos.x, canvasPos.y, radius * scale, 0, 2 * Math.PI);
            this.ctx.fill();
            this.ctx.restore();
        }

        // --- Draw player triangle ---
        let size = self ? GAME_CONSTANTS.DEFAULT_PLAYER_SIZE : GAME_CONSTANTS.OTHER_PLAYER_SIZE;
        let fillColor = self ? GAME_CONSTANTS.COLORS.SELF_PLAYER : GAME_CONSTANTS.COLORS.OTHER_PLAYER;
        let strokeColor = self ? GAME_CONSTANTS.COLORS.SELF_PLAYER_STROKE : GAME_CONSTANTS.COLORS.OTHER_PLAYER_STROKE;

        this.ctx.save();
        this.ctx.translate(canvasPos.x, canvasPos.y);

        let angle = 0;
        if (velocity.x !== 0 || velocity.y !== 0) {
            angle = Math.atan2(velocity.y, velocity.x);
        }
        this.ctx.rotate(angle);

        this.ctx.beginPath();
        this.ctx.moveTo(size, 0);
        this.ctx.lineTo(-size / 2, -size / 2);
        this.ctx.lineTo(-size / 2, size / 2);
        this.ctx.closePath();

        this.ctx.fillStyle = fillColor;
        this.ctx.fill();
        this.ctx.strokeStyle = strokeColor;
        this.ctx.lineWidth = 1;
        this.ctx.stroke();

        // Label for others
        if (!self) {
            this.ctx.fillStyle = GAME_CONSTANTS.COLORS.TEXT;
            this.ctx.font = '10px Arial';
            this.ctx.fillText(player.networkId ?? '', size + 2, -size - 2);
        }

        this.ctx.restore();
    }

    
    render(entities = new Map(), viewDistance = GAME_CONSTANTS.DEFAULT_VIEW_DISTANCE) {
        this.ctx.clearRect(0, 0, this.width, this.height);

        // Extract self and others
        let selfPlayer = null;
        const otherPlayers = [];

        for (const player of entities.values()) {
            // Skip players that aren't initialized
            if (!player.initialized) {
                continue;
            }
            
            if (player.self) selfPlayer = player;
            else otherPlayers.push(player);
        }

        // Draw self player (with view radius etc)
        if (selfPlayer) {
            this.drawPlayer(selfPlayer, {
                viewDistance,
                scale: this.scale,
                canvasWidth: this.width,
                canvasHeight: this.height,
            });
        }

        // Draw other players only if they're within view distance
        if (selfPlayer) {
            const selfPos = selfPlayer.state.position;
            
            for (const player of otherPlayers) {
                const otherPos = player.state.position;
                
                // Calculate distance between self and other player (using Manhattan distance for consistency with server)
                const dx = Math.abs(selfPos.x - otherPos.x);
                const dy = Math.abs(selfPos.y - otherPos.y);
                const distance = dx + dy;
                
                // Only draw if within view distance (with same adjustment factor as server: 1.4)
                if (distance <= viewDistance * this.scale) {
                    this.drawPlayer(player, {});
                }
            }
        } else {
            // If no self player, draw all others (fallback)
            for (const player of otherPlayers) this.drawPlayer(player, {});
        }
    }

}

export default GameRenderer