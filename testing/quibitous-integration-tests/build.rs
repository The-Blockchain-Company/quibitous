fn main() {
    let jor_cli_name = option_env!("JOR_CLI_NAME").unwrap_or("qcli");
    let quibitous_name = option_env!("QUIBITOUS_NAME").unwrap_or("quibitous");
    let jor_explorer_name = option_env!("JOR_EXPLORER_NAME").unwrap_or("explorer");
    println!("cargo:rustc-env=JOR_CLI_NAME={}", jor_cli_name);
    println!("cargo:rustc-env=QUIBITOUS_NAME={}", quibitous_name);
    println!("cargo:rustc-env=JOR_EXPLORER_NAME={}", jor_explorer_name);
    println!("cargo:rustc-env=RUST_BACKTRACE=full");
}
