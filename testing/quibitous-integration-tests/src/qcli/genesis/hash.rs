use quibitous_automation::qcli::QCli;

use assert_fs::prelude::*;
use assert_fs::TempDir;

#[test]
pub fn test_correct_hash_is_returned_for_correct_block() {
    let qcli: QCli = Default::default();
    let content = qcli.genesis().init();
    let temp_dir = TempDir::new().unwrap();
    let yaml_file = temp_dir.child("init_file.yaml");
    yaml_file.write_str(&content).unwrap();
    let block_file = temp_dir.child("block-0.bin");

    qcli.genesis().encode(yaml_file.path(), &block_file);
    qcli.genesis().hash(block_file.path());
}

#[test]
pub fn test_correct_error_is_returned_for_non_existent_genesis_block() {
    let temp_dir = TempDir::new().unwrap();
    let block_file = temp_dir.child("block-0.bin");
    let qcli: QCli = Default::default();
    qcli.genesis().hash_expect_fail(block_file.path(), "file");
}
