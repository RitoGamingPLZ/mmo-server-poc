import { GAME_CONSTANTS, WorldBounds } from './constants.js';

class Player {
    constructor(self = false) {
        this.id = null;
        this.networkId = null;
        this.self = self;
        this.state = {
            position: { x: 0, y: 0 },
            velocity: { x: 0, y: 0 },
            desiredVelocity: { x: 0, y: 0 },
            targetDirection: { x: 0, y: 0 },
        };
        this.color = GAME_CONSTANTS.COLORS.SELF_PLAYER;
        this.size = GAME_CONSTANTS.DEFAULT_PLAYER_SIZE;
    }

    setIds(playerId, networkId) {
        this.id = playerId;
        this.networkId = networkId;
    }

    stop() {
        this.setInputDirection(0, 0);
    }

    setInputDirection(x, y) {
        const st = this.state;
        const mag = Math.sqrt(x * x + y * y);
        if (mag > 0) {
            st.targetDirection.x = x / mag;
            st.targetDirection.y = y / mag;
            st.desiredVelocity.x = st.targetDirection.x * GAME_CONSTANTS.MAX_SPEED;
            st.desiredVelocity.y = st.targetDirection.y * GAME_CONSTANTS.MAX_SPEED;
        } else {
            st.targetDirection.x = 0;
            st.targetDirection.y = 0;
            st.desiredVelocity.x = 0;
            st.desiredVelocity.y = 0;
        }
    }

    tick(dt, worldBounds = null) {
        const st = this.state;
        const MIN = GAME_CONSTANTS.MIN_VELOCITY_THRESHOLD;

        const dx = st.desiredVelocity.x - st.velocity.x;
        const dy = st.desiredVelocity.y - st.velocity.y;
        const moving = Math.abs(st.desiredVelocity.x) > MIN || Math.abs(st.desiredVelocity.y) > MIN;

        if (moving) {
            const lerpFactor = Math.min(GAME_CONSTANTS.ACCELERATION * dt, 1.0);
            st.velocity.x += dx * lerpFactor;
            st.velocity.y += dy * lerpFactor;
        } else {
            const frictionFactor = 1.0 - Math.min(GAME_CONSTANTS.FRICTION * dt, 1.0);
            st.velocity.x *= frictionFactor;
            st.velocity.y *= frictionFactor;
            if (Math.abs(st.velocity.x) < MIN) st.velocity.x = 0;
            if (Math.abs(st.velocity.y) < MIN) st.velocity.y = 0;
        }

        let speedApprox = Math.abs(st.velocity.x) + Math.abs(st.velocity.y);
        const maxSpeed = GAME_CONSTANTS.MAX_SPEED * 1.4;
        if (speedApprox > maxSpeed && speedApprox > 0) {
            const scale = maxSpeed / speedApprox;
            st.velocity.x *= scale;
            st.velocity.y *= scale;
        }

        st.position.x += st.velocity.x * dt;
        st.position.y += st.velocity.y * dt;

        // Boundary collision detection using consolidated logic
        if (worldBounds) {
            const result = WorldBounds.handleBoundaryCollision(st.position, st.velocity, worldBounds);
            st.position = result.position;
            st.velocity = result.velocity;
        }
    }

    setState(field, x, y) {
        if (this.state[field]) {
            this.state[field].x = x;
            this.state[field].y = y;
        } else {
            throw new Error(`Unknown state vector: ${field}`);
        }
    }

    distanceTo(x, y) {
        const dx = x - this.state.position.x;
        const dy = y - this.state.position.y;
        return Math.sqrt(dx * dx + dy * dy);
    }

    isWithinRange(x, y, range) {
        return this.distanceTo(x, y) <= range;
    }

    reset() {
        this.id = null;
        this.networkId = null;
        this.state.position = { x: 0, y: 0 };
        this.state.velocity = { x: 0, y: 0 };
        this.state.targetDirection = { x: 0, y: 0 };
        this.state.desiredVelocity = { x: 0, y: 0 };
    }

    isConnected() {
        return this.id !== null && this.networkId !== null;
    }

    toString() {
        const st = this.state;
        return `Player(id=${this.id}, networkId=${this.networkId}, pos=[${st.position.x.toFixed(1)}, ${st.position.y.toFixed(1)}], vel=[${st.velocity.x.toFixed(2)}, ${st.velocity.y.toFixed(2)}])`;
    }
}

export default Player;
