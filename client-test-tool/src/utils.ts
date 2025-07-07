// Utility functions

// MessagePack decoder (simplified for client-side)
export function decode(data: Uint8Array): any {
  // For now, assume JSON strings are sent as binary
  return JSON.parse(new TextDecoder().decode(data));
}