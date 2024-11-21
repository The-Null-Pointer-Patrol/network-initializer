use std::collections::{HashMap, HashSet};

use crossbeam_channel::{Receiver, Sender,unbounded};
use null_pointer_drone::MyDrone;
use wg_2024::controller::Command;
use wg_2024::drone::{Drone};

// currently is in working group PR
pub struct DroneOptions {
    pub sim_contr_send: Sender<Command>,
    pub sim_contr_recv: Receiver<Command>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    pub pdr: f32,
}

use wg_2024::config::{Config,Drone as DroneCfg,Client as ClientCfg,Server as ServerCfg};
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

fn main() {
    let config_data = std::fs::read_to_string("./input.toml").expect("Unable to read config file");
    let config: Config = toml::from_str(&config_data).expect("Unable to parse TOML");

    println!("{:#?}", config);



    let mut edges : HashSet<(NodeId,NodeId)> = HashSet::new();

    let mut drones :Vec<DroneOptions> = Vec::new();

    let mut simulation_controller_receivers : HashMap<NodeId,Receiver<Command>> = HashMap::new();
    let mut simulation_controller_senders : HashMap<NodeId,Sender<Command>> = HashMap::new();

    let mut tmp_sender_for_node:HashMap<NodeId,Sender<Packet>>=HashMap::new();
    
    for d in config.drone{
        let (drone_send_command,sim_receive_command) = unbounded::<Command>();
        let (sim_send_command,drone_receive_command) = unbounded::<Command>();
        let (drone_send_packet,drone_receive_packet) = unbounded::<Packet>();

        simulation_controller_receivers.insert(d.id, sim_receive_command);
        simulation_controller_senders.insert(d.id,sim_send_command);
        // saves the channel on which the drone receives packets to use it later when creating edges
        tmp_sender_for_node.insert(d.id, drone_send_packet);

        drones.push(DroneOptions{
            sim_contr_send: drone_send_command,
            sim_contr_recv: drone_receive_command,
            packet_recv: drone_receive_packet,
            packet_send: HashMap::new(),
            pdr: d.pdr,
        });
        for e in d.connected_drone_ids {
            edges.insert((d.id, e));
        }
    }

    for c in config.client{
        for e in c.connected_drone_ids {
            edges.insert((c.id, e));
        }
    }

    for s in config.server{
        for e in s.connected_drone_ids {
            edges.insert((s.id, e));
        }
    }

    let mut node_channels : HashMap<NodeId,(Vec<Sender<Packet>>,Receiver<Packet>,Receiver<Packet>)> = HashMap::new();
    let mut simulation_controller_receiver : Receiver<Packet>;

    while let Some((from,to)) = edges.iter().copied().next(){
        if !edges.contains(&(to,from)){
            panic!("initialization file is incorrect, as it does not represent a bidirectional graph: edge from {} to {} but no corresponding edge in node {}",from,to,from);
        }
        let (s,r) = unbounded::<Packet>();

        edges.remove(&(from,to));
        edges.remove(&(to,from));
    }



}
