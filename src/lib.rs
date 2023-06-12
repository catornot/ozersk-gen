#![feature(int_roundings)]

use client::setup_client_concommands;
use compile::compile_map;
use config::setup_config_convars;
use maze::create_maze;

use parking_lot::Mutex;
use rrplug::prelude::*;
use rrplug::wrappers::northstar::{EngineLoadType, PluginData, ScriptVmType};
use server::{register_sq_functions, vm_wait_start};
use std::{path::PathBuf, thread};

mod client;
mod compile;
mod config;
mod maze;
mod mesh;
mod server;

#[derive(Debug)]
pub struct OzerskGenPlugin {
    compiler_path: Mutex<PathBuf>,
    mod_path: Mutex<PathBuf>,
}

impl Plugin for OzerskGenPlugin {
    type SaveType = squirrel::NoSave;

    fn new() -> Self {
        Self {
            compiler_path: Mutex::new(PathBuf::new()),
            mod_path: Mutex::new(PathBuf::new()),
        }
    }

    fn initialize(&mut self, plugin_data: &PluginData) {
        register_sq_functions(plugin_data)
    }

    fn main(&self) {}

    fn on_sqvm_created(&self, sqvm_handle: &squirrel::CSquirrelVMHandle<Self::SaveType>) {
        if sqvm_handle.get_context() == ScriptVmType::Server {
            vm_wait_start(sqvm_handle);

            thread::spawn(|| {
                create_maze( None );
                compile_map(ScriptVmType::Server);
            });
        }
    }

    fn on_engine_load(&self, engine: &EngineLoadType) {
        let engine = match engine {
            EngineLoadType::Engine(engine) => engine,
            EngineLoadType::EngineFailed => return,
            EngineLoadType::Server => return,
            EngineLoadType::Client => return,
        };

        setup_client_concommands(engine);
        setup_config_convars();
    }
}

entry!(OzerskGenPlugin);
