use std::thread;

use ap24_simulation_controller::MySimulationController;
use dummy_nodes::{MyClient, MyServer};
use null_pointer_drone::MyDrone;
use wg_2024::drone::Drone;
// use wg_2024::drone::DroneOptions;

mod config_loader;
mod dummy_nodes;

use wg_2024::config::Config;

fn main() {
    // load config from toml
    let config_data = std::fs::read_to_string("./input.toml").expect("Unable to read config file");
    let config: Config = toml::from_str(&config_data).expect("Unable to parse TOML");

    let (drones, clients, servers, mut simcontr) = config_loader::config_to_options(&config);

    for (_id, options) in drones {
        let handler = thread::spawn(move || {
            let mut drone = MyDrone::new(options);
            drone.run();
        });
        simcontr.node_handles.push(handler);
    }

    for (_id, options) in servers {
        let handler = thread::spawn(move || {
            let server = MyServer::new(options);
            server.run();
        });
        simcontr.node_handles.push(handler);
    }

    for (_id, options) in clients {
        let handler = thread::spawn(move || {
            let client = MyClient::new(options);
            client.run();
        });
        simcontr.node_handles.push(handler);
    }

    let handler = thread::spawn(move || {
        let mut simulation_controller = MySimulationController::new(simcontr);
        simulation_controller.run();
    });

    handler.join();

    println!("Simulation Controller thread exited");
}
