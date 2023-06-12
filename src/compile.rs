use crate::PLUGIN;
use rrplug::wrappers::northstar::ScriptVmType;
use rrplug::{prelude::*, wrappers::squirrel::async_call_sq_function};
use std::process::Command;
use std::{fs, thread};

// also change thing in server/mod.rs
pub const MAP_NAME: &str = "mp_ozersk_maze";

pub fn compile_map(context: ScriptVmType) {
    let compiler = &PLUGIN.wait().compiler_path.lock();
    let basepath = &PLUGIN.wait().mod_path.lock();
    let path = basepath.join(format!("Titanfall2/maps/{MAP_NAME}.map"));

    log::info!("compiling {MAP_NAME}");

    match Command::new(format!("{}", compiler.display()))
        .args([
            "-v".into(),
            "-connect".into(),
            "127.0.0.1:39000".into(),
            "-game".into(),
            "titanfall2".into(),
            "-fs_basepath".into(),
            basepath.display().to_string(),
            "-fs_game".into(),
            "Titanfall2".into(),
            "-meta".into(),
            format!("{}", path.display()),
        ])
        .spawn()
    {
        Ok(child) => {
            _ = thread::spawn(move || match child.wait_with_output() {
                Ok(out) => {
                    log::info!("compilation finished {}", out.status);
                    copy_bsp();

                    async_call_sq_function(context, "OzerskGenCompileDone", None)
                }
                Err(err) => log::error!("compilation failed: command execution fail, {err:?}"),
            })
        }
        Err(err) => log::error!("compilation failed: command not sent, {err:?}"),
    }
}

fn copy_bsp() {
    log::info!("copying bsp to cat_or_not.OzerskGen/mod/maps");

    let path_maps = &PLUGIN.wait().mod_path.lock();

    let files = vec![format!("{MAP_NAME}.bsp"), format!("{MAP_NAME}_script.ent")];

    for file in files.iter() {
        log::info!("copying {file} to {}", path_maps.display());

        match fs::remove_file(path_maps.join(file)) {
            Ok(_) => log::info!("removed old bsp"),
            Err(err) => log::error!("failed to remove old bsp: {err}"),
        }

        match fs::copy(
            path_maps.join(format!("Titanfall2/maps/{file}")),
            path_maps.join(file),
        ) {
            Ok(_) => log::info!("copied bsp to maps folder"),
            Err(err) => log::error!("failed to copy bsp to maps folder: {err}"),
        }
    }
}
