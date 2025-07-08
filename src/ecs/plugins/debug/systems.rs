/*!
# Debug Systems

Development and debugging tools for the game server.

These systems help developers monitor the game state during development.
All debug output can be safely removed in production builds.
*/

use bevy::prelude::*;
use crate::ecs::plugins::player::components::Player;
use crate::ecs::plugins::network::ws::components::ConnectedClients;
use std::time::Duration;
use std::process;
use std::fs;

/// How often to print debug information (in seconds)
const DEBUG_PRINT_INTERVAL: f32 = 1.0;

/// Resource to track when we last printed debug info (replaces unsafe static)
#[derive(Resource, Default)]
pub struct DebugTimer {
    last_print_time: f32,
    last_cpu_time: Option<u64>,
    last_cpu_check: Option<std::time::Instant>,
}

/// Resource to track connection metrics
#[derive(Resource)]
pub struct ConnectionMetrics {
    pub total_connections: u32,
    pub total_disconnections: u32,
    pub peak_concurrent_connections: u32,
    pub server_start_time: std::time::Instant,
}

impl ConnectionMetrics {
    pub fn new() -> Self {
        Self {
            total_connections: 0,
            total_disconnections: 0,
            peak_concurrent_connections: 0,
            server_start_time: std::time::Instant::now(),
        }
    }
    
    pub fn record_connection(&mut self, current_concurrent: u32) {
        self.total_connections += 1;
        if current_concurrent > self.peak_concurrent_connections {
            self.peak_concurrent_connections = current_concurrent;
        }
    }
    
    pub fn record_disconnection(&mut self) {
        self.total_disconnections += 1;
    }
    
    pub fn get_uptime(&self) -> Duration {
        self.server_start_time.elapsed()
    }
}

/// Get current memory usage in MB
fn get_memory_usage() -> f64 {
    let pid = process::id();
    if let Ok(contents) = fs::read_to_string(format!("/proc/{}/status", pid)) {
        for line in contents.lines() {
            if line.starts_with("VmRSS:") {
                if let Some(kb_str) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb_str.parse::<f64>() {
                        return kb / 1024.0; // Convert KB to MB
                    }
                }
            }
        }
    }
    0.0
}

/// Get current CPU time in ticks
fn get_cpu_time() -> Option<u64> {
    let pid = process::id();
    if let Ok(contents) = fs::read_to_string(format!("/proc/{}/stat", pid)) {
        let fields: Vec<&str> = contents.split_whitespace().collect();
        if fields.len() >= 15 {
            // utime (14th field) + stime (15th field) = total CPU time in clock ticks
            if let (Ok(utime), Ok(stime)) = (fields[13].parse::<u64>(), fields[14].parse::<u64>()) {
                return Some(utime + stime);
            }
        }
    }
    None
}

/// Calculate CPU usage percentage based on time difference
fn calculate_cpu_usage(debug_timer: &mut DebugTimer) -> f64 {
    let current_time = std::time::Instant::now();
    let current_cpu_time = get_cpu_time();
    
    if let (Some(current_cpu), Some(last_cpu), Some(last_check)) = 
        (current_cpu_time, debug_timer.last_cpu_time, debug_timer.last_cpu_check) {
        
        let time_diff = current_time.duration_since(last_check).as_secs_f64();
        let cpu_diff = current_cpu.saturating_sub(last_cpu) as f64;
        
        // CPU usage = (cpu_time_diff / real_time_diff) * 100
        // Note: This assumes 100 ticks per second (common on Linux)
        let cpu_usage = if time_diff > 0.0 {
            (cpu_diff / (time_diff * 100.0)) * 100.0
        } else {
            0.0
        };
        
        // Update for next calculation
        debug_timer.last_cpu_time = current_cpu_time;
        debug_timer.last_cpu_check = Some(current_time);
        
        cpu_usage.min(100.0) // Cap at 100%
    } else {
        // First time or error - initialize
        debug_timer.last_cpu_time = current_cpu_time;
        debug_timer.last_cpu_check = Some(current_time);
        0.0
    }
}

/// Debug system that prints game state information every second.
/// 
/// This helps developers see what's happening in the game world:
/// - How many players are connected
/// - Connection metrics and statistics
/// - System resource usage
/// 
/// The system uses a safe Resource instead of unsafe static variables.
pub fn debug_system(
    player_query: Query<&Player>,
    connected_clients: Res<ConnectedClients>,
    connection_metrics: Res<ConnectionMetrics>,
    time: Res<Time>,
    mut debug_timer: ResMut<DebugTimer>,
) {
    let current_time = time.elapsed_secs();
    
    // Only print debug info every DEBUG_PRINT_INTERVAL seconds
    if current_time - debug_timer.last_print_time > DEBUG_PRINT_INTERVAL {
        println!("=== Game State Debug ===");
        
        // Connection metrics
        let current_connections = connected_clients.clients.len() as u32;
        let uptime = connection_metrics.get_uptime();
        let uptime_secs = uptime.as_secs();
        let memory_usage = get_memory_usage();
        let cpu_usage = calculate_cpu_usage(&mut debug_timer);
        
        println!("Connection Metrics:");
        println!("  Current connections: {}", current_connections);
        println!("  Total connections: {}", connection_metrics.total_connections);
        println!("  Total disconnections: {}", connection_metrics.total_disconnections);
        println!("  Peak concurrent: {}", connection_metrics.peak_concurrent_connections);
        println!("  Server uptime: {}h {}m {}s", 
            uptime_secs / 3600, (uptime_secs % 3600) / 60, uptime_secs % 60);
        
        println!("System Resources:");
        println!("  Memory usage: {:.1} MB", memory_usage);
        println!("  CPU usage: {:.1}%", cpu_usage);
        
        if connection_metrics.total_connections > 0 {
            let avg_session_duration = if connection_metrics.total_disconnections > 0 {
                format!("{:.1}s", uptime_secs as f32 / connection_metrics.total_disconnections as f32)
            } else {
                "N/A".to_string()
            };
            println!("  Avg session duration: {}", avg_session_duration);
            
            let connections_per_hour = if uptime_secs > 0 {
                (connection_metrics.total_connections as f32 / uptime_secs as f32) * 3600.0
            } else {
                0.0
            };
            println!("  Connections/hour: {:.1}", connections_per_hour);
        }
        
        // Client connection details
        // if !connected_clients.clients.is_empty() {
        //     println!("\nConnected Clients:");
        //     for (client_id, client_info) in &connected_clients.clients {
        //         let session_duration = client_info.connected_at.elapsed();
        //         match client_id {
        //             crate::ecs::plugins::network::ws::components::ClientId::WebSocket(addr) => {
        //                 println!("  WebSocket {} (connected: {}s ago)", 
        //                     addr, session_duration.as_secs());
        //             }
        //         }
        //     }
        // }
        
        // Player game state
        let player_count = player_query.iter().count();
        
        if player_count == 0 {
            println!("\nNo players in game");
        } else {
            println!("\nActive Players: {}", player_count);
        }
        
        println!("\nServer time: {:.1}s", current_time);
        println!("{}", "=".repeat(50)); // Separator line
        println!(); // Empty line for readability
        
        debug_timer.last_print_time = current_time;
    }
}