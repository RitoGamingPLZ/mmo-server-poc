// Type definitions for the game client

export type InputCommand =
  | {"Move": { direction: [number, number] }}
  | {"Stop": null};

// Network message format matching server's CompactNetworkMessage
export type FieldUpdate = {
  f: string;  // field name (shortened)
  v: any;     // value
};

export type ComponentUpdate = {
  c: string;  // component name (shortened)
  u: FieldUpdate[];  // updates (shortened)
};

export type EntityUpdate = {
  network_id: number;  // network_id from server
  components: { [key: string]: any };  // components object from server
};

// Server's actual format
export type ServerEntityUpdate = {
  network_id: number;
  components: { [key: string]: any };
};

export type CompactNetworkMessage = {
  t: string;  // message_type (shortened)
  es: EntityUpdate[];  // entity_updates (shortened)  
  p: number;  // my_player_id (shortened)
};

// Client-side player state with interpolation
export type Player = {
  id: number;
  
  // Current authoritative state from server
  serverPosition: { x: number; y: number };
  serverVelocity: { x: number; y: number };
  lastServerUpdate: number;
  
  // Client-side simulated state for smooth movement
  position: { x: number; y: number };
  velocity: { x: number; y: number };
  
  // Visual state
  isMyPlayer: boolean;
};