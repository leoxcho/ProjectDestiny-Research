use destiny_activity_service::ActivityService;
use destiny_auth_service::AuthService;
use destiny_inventory_service::InventoryService;
use destiny_network::{Packet, PacketLogger};
use destiny_runtime_core::Runtime;
use destiny_world_service::WorldService;

use destiny_server_framework::{
    ConnectionManager,
    ServiceRegistry,
};

use std::sync::Arc;
use std::time::Instant;

#[derive(Default, Debug, Clone)]
pub struct Telemetry {
    pub request_log: Vec<String>,
    pub service_timing_ms: Vec<(String, u128)>,
    pub packet_traces: Vec<String>,
    pub session_history: Vec<String>,
}

pub struct ServerRuntime {
    pub network: NetworkLayer,
    pub auth: AuthService,
    pub inventory: InventoryService,
    pub activities: ActivityService,
    pub world: WorldService,
    pub telemetry: Telemetry,

    pub connections: ConnectionManager,
    pub services: ServiceRegistry,
}

pub struct NetworkLayer {
    pub log: destiny_network::MemoryPacketLog,
}

impl Default for NetworkLayer {
    fn default() -> Self {
        Self {
            log: Default::default(),
        }
    }
}

impl ServerRuntime {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Self {
            network: NetworkLayer::default(),

            auth: AuthService::default(),
            inventory: InventoryService::new(runtime.clone()),
            activities: ActivityService::new(runtime.clone()),
            world: WorldService::default(),

            telemetry: Telemetry::default(),

            connections: ConnectionManager::default(),
            services: ServiceRegistry::default(),
        }
    }

    pub fn start(&self) {
        tracing::info!("server runtime ready");

        tracing::info!(
            "connections initialized: {}",
            self.connections.connections.lock().unwrap().len()
        );

        tracing::info!("service registry initialized");
    }

    pub fn dispatch(&mut self, packet: Packet) {
        let start = Instant::now();

        let result = if packet.opcode.is_none() {
            "UnknownOpcode"
        } else {
            "Accepted"
        };

        self.telemetry
            .packet_traces
            .push(format!("packet result: {}", result));

        self.telemetry
            .service_timing_ms
            .push(("dispatch".to_string(), start.elapsed().as_millis()));
    }
}
