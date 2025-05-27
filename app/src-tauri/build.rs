fn main() {
    let project_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let env_path = std::path::Path::new(&project_root).join("..").join(".env");

    if env_path.exists() {

        match dotenvy::from_path_iter(&env_path) {
            Ok(iter) => {
                for item in iter {
                    match item {
                        Ok((key, value)) => {
                        }
                        Err(e) => {
                        }
                    }
                }

            }
            Err(e) => {

            }
        }
    } else {

    }

    tauri_build::build();
}
