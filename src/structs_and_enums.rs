use std::collections::HashMap;

use crossbeam_channel::{Receiver, Sender};
use wg_2024::{ controller::Command, network::NodeId, packet::Packet};



// ? should this be up to the group or defined in WG repo?
#[derive(Debug, Clone)]
pub struct ClientServerOptions {
    pub id: NodeId,
    pub sim_contr_send: Sender<Command>,
    pub sim_contr_recv: Receiver<Command>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
}

