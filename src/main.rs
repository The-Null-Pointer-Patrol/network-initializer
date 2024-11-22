use std::collections::{HashMap, HashSet};
use std::thread;

use crossbeam_channel::{unbounded, Receiver, Sender};
use null_pointer_drone::MyDrone;
use wg_2024::controller::{Command, SimulationController};
use wg_2024::drone::Drone;
// use wg_2024::drone::DroneOptions;

mod config_loader;

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

// ? should this be up to the group or defined in WG repo?
#[derive(Debug, Clone)]
pub struct ClientServerOptions {
    pub id: NodeId,
    pub sim_contr_send: Sender<Command>,
    pub sim_contr_recv: Receiver<Command>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
}

pub struct SimControllerOptions {
    pub command_send: HashMap<NodeId, Sender<Command>>,
    pub command_recv: Receiver<Command>,
    // a way to know from id-> nodetype
}

enum NodeKind {
    Drone,
    Server,
    Client,
}

// ? shouldn't this be up to the single groups
use wg_2024::config::{Client as ClientCfg, Config, Drone as DroneCfg, Server as ServerCfg};
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

fn main() {
    let config_data = std::fs::read_to_string("./input.toml").expect("Unable to read config file");
    let config: Config = toml::from_str(&config_data).expect("Unable to parse TOML");

    // println!("{:#?}", config);

    let mut drones: HashMap<NodeId, DroneOptions> = HashMap::new();
    let mut clients: HashMap<NodeId, ClientServerOptions> = HashMap::new();
    let mut servers: HashMap<NodeId, ClientServerOptions> = HashMap::new();
    let (node_command_sender, simcontroller_command_receiver) = unbounded::<Command>();
    let mut simcontr = SimControllerOptions {
        command_send: HashMap::new(),
        command_recv: simcontroller_command_receiver,
    };

    config_loader::config_to_options(
        &config,
        &mut drones,
        &mut clients,
        &mut servers,
        &mut simcontr,
        node_command_sender,
    );

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




