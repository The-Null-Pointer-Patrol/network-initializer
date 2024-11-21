use std::collections::{HashMap, HashSet};
use std::thread;

use crossbeam_channel::{unbounded, Receiver, Sender};
use null_pointer_drone::MyDrone;
use wg_2024::controller::{Command, SimulationController};
use wg_2024::drone::Drone;
// use wg_2024::drone::DroneOptions;

// currently is in working group PR
pub struct DroneOptions {
    pub id: NodeId,
    pub sim_contr_send: Sender<Command>,
    pub sim_contr_recv: Receiver<Command>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    pub pdr: f32,
}

enum NodeKind {
    Drone,
    Server,
    Client,
}

use wg_2024::config::{Client as ClientCfg, Config, Drone as DroneCfg, Server as ServerCfg};
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

fn main() {
    let config_data = std::fs::read_to_string("./input.toml").expect("Unable to read config file");
    let config: Config = toml::from_str(&config_data).expect("Unable to parse TOML");

    // println!("{:#?}", config);

    let mut edges: HashSet<(NodeId, NodeId)> = HashSet::new();

    let mut drones: HashMap<NodeId, DroneOptions> = HashMap::new();

    let mut simulation_controller_receivers: HashMap<NodeId, Receiver<Command>> = HashMap::new();
    let mut simulation_controller_senders: HashMap<NodeId, Sender<Command>> = HashMap::new();

    let mut tmp_sender_for_node: HashMap<NodeId, Sender<Packet>> = HashMap::new();
    let mut tmp_nodekind_identifier: HashMap<NodeId, NodeKind> = HashMap::new();

    for d in config.drone {
        let (drone_send_command, sim_receive_command) = unbounded::<Command>();
        let (sim_send_command, drone_receive_command) = unbounded::<Command>();
        let (drone_send_packet, drone_receive_packet) = unbounded::<Packet>();

        simulation_controller_receivers.insert(d.id, sim_receive_command);
        simulation_controller_senders.insert(d.id, sim_send_command);
        // saves the channel on which the drone receives packets to use it later when creating edges
        tmp_sender_for_node.insert(d.id, drone_send_packet);

        drones.insert(
            d.id,
            DroneOptions {
                id: d.id,
                sim_contr_send: drone_send_command,
                sim_contr_recv: drone_receive_command,
                packet_recv: drone_receive_packet,
                packet_send: HashMap::new(),
                pdr: d.pdr,
            },
        );

        for e in d.connected_drone_ids {
            edges.insert((d.id, e));
        }

        tmp_nodekind_identifier.insert(d.id, NodeKind::Drone);
    }

    for c in config.client {
        for e in c.connected_drone_ids {
            edges.insert((c.id, e));
        }
        tmp_nodekind_identifier.insert(c.id, NodeKind::Client);
    }

    for s in config.server {
        for e in s.connected_drone_ids {
            edges.insert((s.id, e));
        }
        tmp_nodekind_identifier.insert(s.id, NodeKind::Server);
    }

    while let Some((from, to)) = edges.iter().copied().next() {
        // check that edge respects bidirectionality
        if !edges.contains(&(to, from)) {
            //? protocol should specify that we need to do this?
            panic!("initialization file is incorrect, as it does not represent a bidirectional graph: edge from {} to {} but no corresponding edge in node {}",from,to,to);
        }

        // for now checks that edge is made of nodes
        match (
            tmp_nodekind_identifier.get(&to),
            tmp_nodekind_identifier.get(&from),
        ) {
            (Some(NodeKind::Drone), Some(NodeKind::Drone)) => {
                println!("{} {}", from, to);
                // creates connection between nodes of the edge
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
            }
            _ => {}
        }

        edges.remove(&(from, to));
        edges.remove(&(to, from));
    }

    let handles = vec![];

    for (id, options) in drones {
        // for now incompatible
        let handler = thread::spawn(move || {
            let mut drone = MyDrone::new(options);
            drone.run();
        });
        // todo: handle result
        handles.push(handler);
    }

    let handler = thread::spawn(move || {
        let mut simulation_controller = todo!();
        simulation_controller.run();
    });
    handles.push(handler);

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    println!("All threads have completed.");

}
