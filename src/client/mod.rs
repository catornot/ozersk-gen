use std::thread;

use base64::{engine::general_purpose, Engine};
use parking_lot::Mutex;
use rrplug::{
    bindings::convar::{FCVAR_CLIENTDLL, FCVAR_SERVER_CAN_EXECUTE},
    wrappers::{engine::EngineData, northstar::ScriptVmType},
};

use crate::{
    compile::{compile_map},
    maze::create_maze,
};

static RECEIVED_MAP_SLICES: Mutex<String> = Mutex::new(String::new());

pub fn setup_client_concommands(engine: &EngineData) {
    engine
        .register_concommand(
            "og_get_map_data",
            parse_received_map_data,
            "",
            FCVAR_SERVER_CAN_EXECUTE as i32 | FCVAR_CLIENTDLL as i32,
        )
        .unwrap();
}

#[rrplug::concommand]
fn parse_received_map_data(command: CCommandResult) {
    if command.args.len() > 2 {
        log::error!("too many args for {}", command.command)
    }

    if let Some(keyword) = command.args.get(0) {
        if keyword == "START" {
            RECEIVED_MAP_SLICES.lock().clear();
        } else if keyword == "END" {
            let lock = RECEIVED_MAP_SLICES.lock();
            let byte_data = lock.as_bytes();

            let mut buf: Vec<_> = Vec::new();
            if let Err(err) = general_purpose::STANDARD_NO_PAD.decode_vec(byte_data, &mut buf) {
                log::info!("can't parse this stuff : {err}");
                return;
            }

            let maze_info = match bincode::deserialize(&buf) {
                Ok(info) => info,
                Err(err) => {
                    log::error!("failed to deserialize stream : {err}");
                    return;
                }
            };

            thread::spawn(move || {
                create_maze(Some(maze_info));
                compile_map(ScriptVmType::Client);
            });
        } else {
            log::info!("got slice of map file");
            RECEIVED_MAP_SLICES.lock().push_str(keyword)
        };
    }
}
