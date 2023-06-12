use std::path::PathBuf;

use once_cell::sync::OnceCell;
use rrplug::{
    bindings::convar::FCVAR_CLIENTDLL,
    wrappers::convars::{ConVarRegister, ConVarStruct},
};

static CONVAR_COMPILER: OnceCell<ConVarStruct> = OnceCell::new();
static CONVAR_MOD: OnceCell<ConVarStruct> = OnceCell::new();

pub fn setup_config_convars() {
    let convar = ConVarStruct::try_new().unwrap();
    convar
        .register(ConVarRegister {
            callback: Some(compiler_path_changed),
            ..ConVarRegister::mandatory(
                "og_compiler_path",
                "/",
                FCVAR_CLIENTDLL as i32,
                "path to the remap compiler",
            )
        })
        .unwrap();

    _ = CONVAR_COMPILER.set(convar);

    let convar = ConVarStruct::try_new().unwrap();
    convar
        .register(ConVarRegister {
            callback: Some(mod_path_changed),
            ..ConVarRegister::mandatory(
                "og_mod_path",
                "/",
                FCVAR_CLIENTDLL as i32,
                "path to the ozersk-gen maps folder",
            )
        })
        .unwrap();

    _ = CONVAR_MOD.set(convar);
}

#[rrplug::convar]
fn compiler_path_changed(convar: Option<ConVarStruct>, old_value: String, float_old_value: f32) {
    let convar = CONVAR_COMPILER.wait();
    let new_path = match convar.get_value().value {
        Some(p) => p,
        None => {
            log::error!("failed to get string value for compiler path");
            return;
        }
    };

    log::info!("replacing compiler path : {} with {}", old_value, new_path);

    let mut lock = crate::PLUGIN.wait().compiler_path.lock();
    *lock = PathBuf::from(new_path);
}

#[rrplug::convar]
fn mod_path_changed(convar: Option<ConVarStruct>, old_value: String, float_old_value: f32) {
    let convar = CONVAR_MOD.wait();
    let new_path = match convar.get_value().value {
        Some(p) => p,
        None => {
            log::error!("failed to get string value for mod path");
            return;
        }
    };

    log::info!("replacing mod path : {} with {}", old_value, new_path);

    let mut lock = crate::PLUGIN.wait().mod_path.lock();
    *lock = PathBuf::from(new_path);
}
