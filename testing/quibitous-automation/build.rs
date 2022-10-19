fn main() {
    tonic_build::compile_protos("proto/node.proto").unwrap();
    tonic_build::compile_protos("proto/watch.proto").unwrap();

    let qui_cli_name = option_env!("QUI_CLI_NAME").unwrap_or("qcli");
    let quibitous_name = option_env!("QUIBITOUS_NAME").unwrap_or("quibitous");
    let qui_explorer_name = option_env!("QUI_EXPLORER_NAME").unwrap_or("explorer");
    println!("cargo:rustc-env=QUI_CLI_NAME={}", qui_cli_name);
    println!("cargo:rustc-env=QUIBITOUS_NAME={}", quibitous_name);
    println!("cargo:rustc-env=QUI_EXPLORER_NAME={}", qui_explorer_name);
    println!("cargo:rustc-env=RUST_BACKTRACE=full");
}
