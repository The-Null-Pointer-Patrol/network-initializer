use null_pointer_drone::MyDrone;
use wg_2024::drone::{Drone, DroneOptions};

use wg_2024::config::Config;

fn main() {
    println!("Hello, world!");
    let config_data = std::fs::read_to_string("input.toml").expect("Unable to read config file");
    let config: Config = toml::from_str(&config_data).expect("Unable to parse TOML");

    // trait Drone is not object safe (because of the new function)
    // let drones : Vec<Box<dyn Drone>> = Vec::new();

    for s in config.drone{

    }

    for c in config.client{

    }

    for s in config.server{

    }

    //println!("{:#?}", config);
}
