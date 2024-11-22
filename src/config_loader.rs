pub fn config_to_options(
    config: &Config,
    drones: &mut HashMap<NodeId, DroneOptions>,
    clients: &mut HashMap<NodeId, ClientServerOptions>,
    servers: &mut HashMap<NodeId, ClientServerOptions>,
    simcontr: &mut SimControllerOptions,
    node_command_sender: Sender<Command>,
) {
    let mut edges: HashSet<(NodeId, NodeId)> = HashSet::new();

    // let mut simulation_controller_receivers: HashMap<NodeId, Receiver<Command>> = HashMap::new();
    let mut simulation_controller_senders: HashMap<NodeId, Sender<Command>> = HashMap::new();

    let mut tmp_sender_for_node: HashMap<NodeId, Sender<Packet>> = HashMap::new();
    let mut tmp_command_: HashMap<NodeId, Sender<Packet>> = HashMap::new();
    let mut tmp_nodekind_identifier: HashMap<NodeId, NodeKind> = HashMap::new();

    for d in config.drone.iter() {
        let (sim_send_command, node_receive_command) = unbounded::<Command>();
        let (drone_send_packet, node_receive_packet) = unbounded::<Packet>();

        simulation_controller_senders.insert(d.id, sim_send_command);
        // saves the channel on which the drone receives packets to use it later when creating edges
        tmp_sender_for_node.insert(d.id, drone_send_packet);

        drones.insert(
            d.id,
            DroneOptions {
                id: d.id,
                sim_contr_send: node_command_sender.clone(),
                sim_contr_recv: node_receive_command,
                packet_recv: node_receive_packet,
                packet_send: HashMap::new(),
                pdr: d.pdr,
            },
        );

        for e in d.connected_drone_ids.iter() {
            edges.insert((d.id, *e));
        }

        tmp_nodekind_identifier.insert(d.id, NodeKind::Drone);
    }

    // for c in config.client {
    //     let (sim_send_command, client_receive_command) = unbounded::<Command>();
    //     let (drone_send_packet, drone_receive_packet) = unbounded::<Packet>();

    //     // simulation_controller_receivers.insert(c.id, client_receive_command);
    //     simulation_controller_senders.insert(c.id, client_send_command);

    //     clients.insert(c.id, ClientServerOptions{
    //         id: c.id,
    //         sim_contr_send: client_send_command,
    //         sim_contr_recv: client_receive_command,
    //         packet_recv: todo!(),
    //     })

    //     for e in c.connected_drone_ids {
    //         edges.insert((c.id, e));
    //     }
    //     tmp_nodekind_identifier.insert(c.id, NodeKind::Client);
    // }

    // for s in config.server {
    //     for e in s.connected_drone_ids {
    //         edges.insert((s.id, e));
    //     }
    //     tmp_nodekind_identifier.insert(s.id, NodeKind::Server);
    // }

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
}