use std::path::PathBuf;

fn config_path() -> PathBuf {
    home::home_dir().unwrap_or("./".into()).join(".radio_rebelde")
}

pub fn load() -> Result<Vec<String>, std::io::Error> {
        let config = std::fs::read_to_string(&config_path()).unwrap_or("[]".to_string());
        let streams_collection: Vec<String> = serde_json::from_str(&config)?;
        Ok(streams_collection)
}

pub fn save(streams_collection: Vec<String>) -> Result<(), std::io::Error> {
    let string_content = serde_json::to_string(&streams_collection)?;
    std::fs::write(config_path(), string_content)?;
    Ok(())
}
