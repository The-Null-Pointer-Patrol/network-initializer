use std::{collections::{HashMap, HashSet}, fmt};

use ap24_simulation_controller::SimControllerOptions;
use crossbeam_channel::{unbounded, Sender};
use wg_2024::{
    config::Config,
    controller::{Command},
    drone::DroneOptions,
    network::NodeId,
    packet::Packet,
};

enum NodeKind{
    Drone,
    Server,
    Client
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeKind::Drone => write!(f, "Drone"),
            NodeKind::Server => write!(f, "Server"),
            NodeKind::Client => write!(f, "Client"),
        }
    }
}

use crate::dummy_nodes::ClientServerOptions;

pub fn config_to_options(
    config: &Config,
) -> (
    HashMap<NodeId, DroneOptions>,
    HashMap<NodeId, ClientServerOptions>,
    HashMap<NodeId, ClientServerOptions>,
    SimControllerOptions,
) {
    let mut drones: HashMap<NodeId, DroneOptions> = HashMap::new();
    let mut clients: HashMap<NodeId, ClientServerOptions> = HashMap::new();
    let mut servers: HashMap<NodeId, ClientServerOptions> = HashMap::new();
    let (node_command_sender, simcontroller_command_receiver) = unbounded::<Command>();
    let mut simcontr = SimControllerOptions {
        command_send: HashMap::new(),
        command_recv: simcontroller_command_receiver,
        packet_send: HashMap::new(),
        config: config.clone(),
    };

    let mut edges: HashSet<(NodeId, NodeId)> = HashSet::new();

    let mut tmp_node_kind : HashMap<NodeId,NodeKind> = HashMap::new();
    let mut tmp_node_ids_set : HashSet<NodeId> = HashSet::new();

    let mut check_id_uniqueness = |id:NodeId| if tmp_node_ids_set.contains(&id) {panic!("error in toml config: two nodes with id {}",id)} else { tmp_node_ids_set.insert(id)};
    
    // the 3 following for cycles have a lot of shared code,
    // but I think it's better to wait to see if there are protocol changes before trying to optimize them

    // todo: check that there are no duplicate ids
    for n in config.drone.iter() {
        check_id_uniqueness(n.id);
        tmp_node_kind.insert(n.id, NodeKind::Drone);

        let (sim_send_command, node_receive_command) = unbounded::<Command>();
        let (node_send_packet, node_receive_packet) = unbounded::<Packet>();

        simcontr.command_send.insert(n.id, sim_send_command);
        // saves the channel on which the drone receives packets to use it later when creating edges
        simcontr.packet_send.insert(n.id, node_send_packet);

        drones.insert(
            n.id,
            DroneOptions {
                id: n.id,
                sim_contr_send: node_command_sender.clone(),
                sim_contr_recv: node_receive_command,
                packet_recv: node_receive_packet,
                packet_send: HashMap::new(),
                pdr: n.pdr,
            },
        );

        for e in n.connected_node_ids.iter() {
            edges.insert((n.id, *e));
        }
    }

    for n in config.client.iter() {
        check_id_uniqueness(n.id);
        tmp_node_kind.insert(n.id, NodeKind::Client);

        let (sim_send_command, node_receive_command) = unbounded::<Command>();
        let (node_send_packet, node_receive_packet) = unbounded::<Packet>();

        simcontr.command_send.insert(n.id, sim_send_command);
        // saves the channel on which the drone receives packets to use it later when creating edges
        simcontr.packet_send.insert(n.id, node_send_packet);

        clients.insert(
            n.id,
            ClientServerOptions {
                id: n.id,
                sim_contr_send: node_command_sender.clone(),
                sim_contr_recv: node_receive_command,
                packet_recv: node_receive_packet,
                packet_send: HashMap::new(),
            },
        );

        for e in n.connected_drone_ids.iter() {
            edges.insert((n.id, *e));
        }
    }

    for n in config.server.iter() {
        check_id_uniqueness(n.id);
        tmp_node_kind.insert(n.id, NodeKind::Server);

        let (sim_send_command, node_receive_command) = unbounded::<Command>();
        let (node_send_packet, node_receive_packet) = unbounded::<Packet>();

        simcontr.command_send.insert(n.id, sim_send_command);
        // saves the channel on which the drone receives packets to use it later when creating edges
        simcontr.packet_send.insert(n.id, node_send_packet);

        servers.insert(
            n.id,
            ClientServerOptions {
                id: n.id,
                sim_contr_send: node_command_sender.clone(),
                sim_contr_recv: node_receive_command,
                packet_recv: node_receive_packet,
                packet_send: HashMap::new(),
            },
        );

        for e in n.connected_drone_ids.iter() {
            edges.insert((n.id, *e));
        }
    }

    let get_kind_from_id = |id:NodeId| if drones.contains_key(&id) {NodeKind::Drone} else if clients.contains_key(&id) {NodeKind::Client} else {NodeKind::Server};

    while let Some((from, to)) = edges.iter().copied().next() {
        // check that edge respects bidirectionality
        if !edges.contains(&(to, from)) {
            //? protocol should specify that we need to do this?
            panic!("initialization file is incorrect, as it does not represent a bidirectional graph: edge from {} to {} but no corresponding edge in node {}",from,to,to);
        }

        if let (Some(from_kind), Some(to_kind)) = (tmp_node_kind.get(&from),tmp_node_kind.get(&to)){
            if !matches!(from_kind, NodeKind::Drone) && !matches!(to_kind, NodeKind::Drone) {
                panic!("Initialization file is incorrect, as there is an edge between {} and {}, which are of type {} and {}, which isn't allowed", from, to, from_kind, to_kind);
            } 
        }  
        // todo: do we need to consider the else case? 
        // considering that in this function it should(?) never happen
        
        // creates the two connections between nodes of the edge
        add_sender_to_node(&mut drones, &mut clients, &mut servers, &from,to, simcontr.packet_send.get(&to).unwrap().clone());
        add_sender_to_node(&mut drones, &mut clients, &mut servers,&to, from, simcontr.packet_send.get(&from).unwrap().clone());

        // remove both representations of edge
        edges.remove(&(from, to));
        edges.remove(&(to, from));
    }

    (drones, clients, servers, simcontr)
}

fn add_sender_to_node(
    drones: &mut HashMap<NodeId, DroneOptions>,
    clients: &mut HashMap<NodeId, ClientServerOptions>,
    servers: &mut HashMap<NodeId, ClientServerOptions>,
    node:&NodeId,
    k:NodeId,
    v:Sender<Packet>
) {
    if drones.contains_key(node){
        drones
            .get_mut(node)
            .unwrap()
            .packet_send
            .insert(k, v);
    }
    else if clients.contains_key(node){
        clients
            .get_mut(node)
            .unwrap()
            .packet_send
            .insert(k, v);
    }
    else if servers.contains_key(node){
        servers
            .get_mut(node)
            .unwrap()
            .packet_send
            .insert(k, v);
    }
}