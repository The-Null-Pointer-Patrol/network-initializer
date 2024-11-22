use std::collections::{HashMap, HashSet};

use crossbeam_channel::{unbounded, Sender};
use wg_2024::{config::Config, controller::Command, network::NodeId, packet::Packet};

use crate::structs_and_enums::{ClientServerOptions, DroneOptions};

use network_initializer::unofficial_wg_implementations::SimControllerOptions;

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
        config:config.clone()
    };

    let mut edges: HashSet<(NodeId, NodeId)> = HashSet::new();

    let mut tmp_sender_for_node: HashMap<NodeId, Sender<Packet>> = HashMap::new();
    // could be added to simulation controller

    // the 3 following for cycles have a lot of shared code,
    // but I think it's better to wait to see if there are protocol changes before trying to optimize them

    // todo: check that there are no duplicate ids
    for n in config.drone.iter() {
        let (sim_send_command, node_receive_command) = unbounded::<Command>();
        let (node_send_packet, node_receive_packet) = unbounded::<Packet>();

        simcontr.command_send.insert(n.id, sim_send_command);
        // saves the channel on which the drone receives packets to use it later when creating edges
        tmp_sender_for_node.insert(n.id, node_send_packet);

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

        for e in n.connected_drone_ids.iter() {
            edges.insert((n.id, *e));
        }

    }

    for n in config.client.iter() {
        let (sim_send_command, node_receive_command) = unbounded::<Command>();
        let (node_send_packet, node_receive_packet) = unbounded::<Packet>();

        simcontr.command_send.insert(n.id, sim_send_command);
        // saves the channel on which the drone receives packets to use it later when creating edges
        tmp_sender_for_node.insert(n.id, node_send_packet);

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
        let (sim_send_command, node_receive_command) = unbounded::<Command>();
        let (node_send_packet, node_receive_packet) = unbounded::<Packet>();

        simcontr.command_send.insert(n.id, sim_send_command);
        // saves the channel on which the drone receives packets to use it later when creating edges
        tmp_sender_for_node.insert(n.id, node_send_packet);

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

    while let Some((from, to)) = edges.iter().copied().next() {
        // check that edge respects bidirectionality
        if !edges.contains(&(to, from)) {
            //? protocol should specify that we need to do this?
            panic!("initialization file is incorrect, as it does not represent a bidirectional graph: edge from {} to {} but no corresponding edge in node {}",from,to,to);
        }

        // creates the two connections between nodes of the edge
        drones
            .get_mut(&from)
            .unwrap()
            .packet_send
            .insert(to, tmp_sender_for_node.get(&to).unwrap().clone());
        drones
            .get_mut(&to)
            .unwrap()
            .packet_send
            .insert(from, tmp_sender_for_node.get(&from).unwrap().clone());

        // remove both representations of edge
        edges.remove(&(from, to));
        edges.remove(&(to, from));
    }

    (drones, clients, servers, simcontr)
}
