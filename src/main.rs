use std::collections::{HashMap, HashSet};
use std::thread;

use ap24_simulation_controller::MySimulationController;
use config_loader::config_to_options;
use crossbeam_channel::{unbounded, Receiver, Sender};
use null_pointer_drone::MyDrone;
use wg_2024::controller::{Command, SimulationController};
use wg_2024::drone::Drone;
// use wg_2024::drone::DroneOptions;

mod config_loader;
mod structs_and_enums;

// ? shouldn't this be up to the single groups
use wg_2024::config::{Client as ClientCfg, Config, Drone as DroneCfg, Server as ServerCfg};

fn main() {
    // load config from toml
    let config_data = std::fs::read_to_string("./input.toml").expect("Unable to read config file");
    let config: Config = toml::from_str(&config_data).expect("Unable to parse TOML");

    let (drones, clients, servers, simcontr) = config_loader::config_to_options(&config);

    let mut handles = vec![];

    for (id, options) in drones {
        // for now incompatible
        let handler = thread::spawn(move || {
            let mut drone = MyDrone::new(options);
            drone.run();
        });
        // todo: handle result
        handles.push(handler);
    }

    for (id, options) in servers {
        // for now incompatible
        let handler = thread::spawn(move || {
            //let mut server = MyServer::new(options);
            //drone.run();
        });
        // todo: handle result
        handles.push(handler);
    }

    for (id, options) in clients {
        // for now incompatible
        let handler = thread::spawn(move || {
            //let mut client = MyClient::new(options);
            //drone.run();
        });
        // todo: handle result
        handles.push(handler);
    }

    let handler = thread::spawn(move || {
        let mut simulation_controller = MySimulationController::new(simcontr);
        simulation_controller.run();
    });
    handles.push(handler);

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    println!("All threads have completed.");
}
