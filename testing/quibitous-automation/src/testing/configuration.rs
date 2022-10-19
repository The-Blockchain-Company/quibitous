use std::env;
use std::path::PathBuf;

pub use crate::quibitous::{
    Block0ConfigurationBuilder, QuibitousParams, NodeConfigBuilder, SecretModelFactory,
    TestConfig,
};

/// Get quibitous executable from current environment
pub fn get_quibitous_app() -> PathBuf {
    const QUIBITOUS_NAME: &str = env!("QUIBITOUS_NAME");
    get_app_from_current_dir(QUIBITOUS_NAME)
}

/// Get qcli executable from current environment
pub fn get_qcli_app() -> PathBuf {
    const QUI_CLI_NAME: &str = env!("QUI_CLI_NAME");
    get_app_from_current_dir(QUI_CLI_NAME)
}

/// Get explorer executable from current environment
pub fn get_explorer_app() -> PathBuf {
    const QUI_EXPLORER_NAME: &str = env!("QUI_EXPLORER_NAME");
    get_app_from_current_dir(QUI_EXPLORER_NAME)
}

/// Get executable from current environment
pub fn get_app_from_current_dir(app_name: &str) -> PathBuf {
    let mut path = get_working_directory();
    path.push(app_name);
    if cfg!(windows) {
        path.set_extension("exe");
    }
    assert!(
        path.is_file(),
        "File does not exist: {:?}, pwd: {:?}",
        path,
        env::current_dir()
    );
    path
}

/// Gets working directory
/// Uses std::env::current_exe() for this purpose.
/// Current exe directory is ./target/{profile}/deps/{app_name}.exe
/// Function returns ./target/{profile}
fn get_working_directory() -> PathBuf {
    let mut output_directory: PathBuf = std::env::current_exe().unwrap();

    output_directory.pop();

    if output_directory.ends_with("deps") {
        output_directory.pop();
    }
    output_directory
}

pub fn get_openapi_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("doc");
    path.push("api");
    path.push("v0.yaml");
    path
}
