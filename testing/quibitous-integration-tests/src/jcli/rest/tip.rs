use quibitous_automation::{
    qcli::QCli,
    quibitous::{NodeConfigBuilder, Starter},
};

#[test]
pub fn test_correct_id_is_returned_for_block_tip_if_only_genesis_block_exists() {
    let qcli: QCli = Default::default();
    let quibitous = Starter::new().start().unwrap();
    let block_id = qcli.rest().v0().tip(quibitous.rest_uri());

    assert_ne!(&block_id, "", "empty block hash");
}

#[test]
pub fn test_correct_error_is_returned_for_incorrect_path() {
    let qcli: QCli = Default::default();
    let config = NodeConfigBuilder::new().build();
    let incorrect_uri = format!("http://{}/api/api", config.rest.listen);

    qcli.rest()
        .v0()
        .tip_expect_fail(incorrect_uri, "tcp connect error");
}
