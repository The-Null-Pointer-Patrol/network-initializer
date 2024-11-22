// simulates structs, traits etc that could be published in the WG repo (hopefully)
// Simcontroller will probably be implemented like this in the next days, 
// things that don't get implemented by wg will be moved to structs_and_enums probably

use std::collections::HashMap;

use crossbeam_channel::{Receiver, Sender};
use wg_2024::{config::Config, controller::Command, network::NodeId, packet::Packet};

pub struct SimControllerOptions {
    pub command_send: HashMap<NodeId, Sender<Command>>,
    pub command_recv: Receiver<Command>,
    pub config: Config,
}

pub trait SimulationController{
    fn new(opt:SimControllerOptions)-> Self;
    fn run(&mut self);
}

// currently is in working group PR
#[derive(Debug, Clone)]

pub struct DroneOptions {
    pub id: NodeId,
    pub sim_contr_send: Sender<Command>,
    pub sim_contr_recv: Receiver<Command>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    pub pdr: f32,
}