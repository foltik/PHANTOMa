use std::sync::{Arc, Mutex};

use crate::component::ComponentState;

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        println!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        println!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn buffer_size(&mut self, _: &jack::Client, sz: jack::Frames) -> jack::Control {
        println!("JACK: buffer size changed to {}", sz);
        jack::Control::Continue
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        println!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        println!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        println!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        println!(
            "JACK: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        println!(
            "JACK: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected {
                "connected"
            } else {
                "disconnected"
            }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: xrun occurred");
        jack::Control::Continue
    }

    fn latency(&mut self, _: &jack::Client, mode: jack::LatencyType) {
        println!(
            "JACK: {} latency has changed",
            match mode {
                jack::LatencyType::Capture => "capture",
                jack::LatencyType::Playback => "playback",
            }
        );
    }
}

pub fn init(state: Arc<Mutex<ComponentState>>) -> jack::AsyncClient<impl jack::NotificationHandler, impl jack::ProcessHandler> {
    let (client, _status) =
        jack::Client::new("PHANTOMa", jack::ClientOptions::NO_START_SERVER).unwrap();

    let in_a = client
        .register_port("in_left", jack::AudioIn::default())
        .unwrap();
    let in_b = client
        .register_port("in_right", jack::AudioIn::default())
        .unwrap();
    /*
    let mut out_a = client
        .register_port("rust_out_l", jack::AudioOut::default())
        .unwrap();
    let mut out_b = client
        .register_port("rust_out_r", jack::AudioOut::default())
        .unwrap();
    */

    let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        //let out_a_p = out_a.as_mut_slice(ps);
        //let out_b_p = out_b.as_mut_slice(ps);
        //let in_a_p = in_a.as_slice(ps);
        //let in_b_p = in_b.as_slice(ps);
        //out_a_p.clone_from_slice(&in_a_p);
        //out_b_p.clone_from_slice(&in_b_p);
        jack::Control::Continue
    };
    let process = jack::ClosureProcessHandler::new(process_callback);

    let active_client = client.activate_async(Notifications, process).unwrap();

    active_client
}

