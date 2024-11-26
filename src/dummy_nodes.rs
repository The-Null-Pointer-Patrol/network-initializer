use std::collections::HashMap;

use crossbeam_channel::{Receiver, Sender};
use wg_2024::{controller::NodeEvent, network::NodeId, packet::Packet};

// ? should this be up to the group or defined in WG repo?
#[derive(Debug, Clone)]
pub struct ClientServerOptions {
    pub id: NodeId,
    pub sim_contr_send: Sender<NodeEvent>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
}

pub struct MyClient {
    options: ClientServerOptions,
}

impl MyClient {
    pub fn new(options: ClientServerOptions) -> Self {
        Self { options }
    }

    pub fn run(self) {
        todo!()
    }
}

pub struct MyServer {
    options: ClientServerOptions,
}

impl MyServer {
    pub fn new(options: ClientServerOptions) -> Self {
        Self { options }
    }

    pub fn run(self) {
        todo!()
    }
}
