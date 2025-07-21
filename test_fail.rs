use std::path::PathBuf;

fn main() {
    env_logger::init();
    
    let manifest_path = PathBuf::from("fail.manifest");
    println!("Attempting to parse: {:?}", manifest_path);
    
    match egdata_manifests_parser::load(&manifest_path) {
        Ok(manifest) => {
            println!("✅ Successfully parsed manifest!");
            println!("Header version: {}", manifest.header.version);
            if let Some(meta) = &manifest.meta {
                println!("App name: {}", meta.app_name);
            }
        }
        Err(e) => {
            println!("❌ Failed to parse manifest: {}", e);
            println!("Error details: {:?}", e);
        }
    }
}