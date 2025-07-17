// Game constants shared across the client
export const GAME_CONSTANTS = {
    // Player movement constants
    MIN_VELOCITY_THRESHOLD: 0.5,
    ACCELERATION: 14.0,
    FRICTION: 10.0,
    MAX_SPEED: 100.0,
    
    // World bounds constants
    WORLD_MIN_X: 0.0,
    WORLD_MIN_Y: 0.0,
    DEFAULT_WORLD_BOUNDS: { x: 1000, y: 1000 },
    
    // Player constants
    DEFAULT_PLAYER_SIZE: 12,
    OTHER_PLAYER_SIZE: 8,
    
    // Rendering constants
    DEFAULT_VIEW_DISTANCE: 300,
    DEFAULT_SCALE: 1,
    MAX_CANVAS_SIZE: 1000,
    
    // Colors
    COLORS: {
        SELF_PLAYER: '#007bff',
        SELF_PLAYER_STROKE: '#0056b3',
        OTHER_PLAYER: '#28a745',
        OTHER_PLAYER_STROKE: '#1e7e34',
        VIEW_RADIUS: '#007bff',
        GRAY_MASK: 'rgba(128, 128, 128, 0.3)',
        TEXT: '#000'
    }
};

// Helper functions for world bounds
export const WorldBounds = {
    isOutOfBounds(position, bounds) {
        return position.x < GAME_CONSTANTS.WORLD_MIN_X || 
               position.x > bounds.x || 
               position.y < GAME_CONSTANTS.WORLD_MIN_Y || 
               position.y > bounds.y;
    },
    
    clampPosition(position, bounds) {
        return {
            x: Math.max(GAME_CONSTANTS.WORLD_MIN_X, Math.min(bounds.x, position.x)),
            y: Math.max(GAME_CONSTANTS.WORLD_MIN_Y, Math.min(bounds.y, position.y))
        };
    },
    
    handleBoundaryCollision(position, velocity, bounds) {
        let newPosition = { ...position };
        let newVelocity = { ...velocity };
        
        if (position.x < GAME_CONSTANTS.WORLD_MIN_X) {
            newPosition.x = GAME_CONSTANTS.WORLD_MIN_X;
            newVelocity.x = -velocity.x;
        } else if (position.x > bounds.x) {
            newPosition.x = bounds.x;
            newVelocity.x = -velocity.x;
        }
        
        if (position.y < GAME_CONSTANTS.WORLD_MIN_Y) {
            newPosition.y = GAME_CONSTANTS.WORLD_MIN_Y;
            newVelocity.y = -velocity.y;
        } else if (position.y > bounds.y) {
            newPosition.y = bounds.y;
            newVelocity.y = -velocity.y;
        }
        
        return { position: newPosition, velocity: newVelocity };
    }
};