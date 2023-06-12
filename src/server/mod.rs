use rrplug::{
    sq_return_notnull, sq_return_null,
    wrappers::{
        northstar::PluginData,
        squirrel::{compile_string, push_sq_array, CSquirrelVMHandle, SQFUNCTIONS},
    },
};

use crate::maze::MAP_FILE_BASE_64;

pub fn register_sq_functions(plugin_data: &PluginData) {
    plugin_data.register_sq_functions(info_on_vm_start);
    plugin_data.register_sq_functions(info_get_map_file_data);
}

pub fn vm_wait_start<T>(cs_sqvm: &CSquirrelVMHandle<T>) {
    let sqvm = unsafe { cs_sqvm.get_sqvm() };
    let sqfunctions = SQFUNCTIONS.server.wait();

    compile_string(
        sqvm,
        sqfunctions,
        true,
        r#"
    thread void function() {
        wait 0
        OzerskGenVMFullStart()
    }()
    "#,
    )
    .unwrap();
}

#[rrplug::sqfunction(VM=Server,ExportName=OzerskGenVMFullStart)]
fn on_vm_start() {
    compile_string(
        sqvm,
        sq_functions,
        true,
        r#"
    
    if ( GetMapName() != "mp_ozersk_maze" )
		return

    ClassicMP_SetLevelIntro( ClassicMP_DefaultNoIntro_Setup, ClassicMP_DefaultNoIntro_GetLength() )
    
    
    AddCallback_OnClientConnected( void function( entity player ) {

        if ( GetPlayerArray()[0] == player && !NSIsDedicated() )
            return
        
        void functionref( entity ) send_data = void function( entity player ) {
            ClientCommand( player, "og_get_map_data START" )
            wait 0
    
            foreach( string data in OzerskGenGetMapData() )
            {
                ClientCommand( player, "og_get_map_data " + data )
                wait 0
            }
    
            ClientCommand( player, "og_get_map_data END" )
        }

        // don't forget about player0
        thread send_data( player )
    } )
    "#,
    )
    .unwrap();

    sq_return_null!();
}

#[rrplug::sqfunction(VM=Server,ExportName=OzerskGenGetMapData)]
fn get_map_file_data() -> Vec<String> {
    let map_data = MAP_FILE_BASE_64.lock();

    push_sq_array(sqvm, sq_functions, map_data.to_vec());

    sq_return_notnull!();
}
