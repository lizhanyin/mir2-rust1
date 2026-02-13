use std::{collections::HashMap, path::PathBuf};

fn main() {
    let library = HashMap::from([("lucide".to_string(), PathBuf::from(lucide_slint::lib()))]);
    let config = slint_build::CompilerConfiguration::new().with_library_paths(library);

    // Specify your Slint code entry here
    slint_build::compile_with_config("ui/app_window.slint", config).expect("Slint build failed");
}
