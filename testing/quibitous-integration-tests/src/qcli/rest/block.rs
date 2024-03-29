use quibitous_automation::{qcli::QCli, quibitous::Starter};

#[test]
pub fn test_non_empty_hash_is_returned_for_block0() {
    let qcli: QCli = Default::default();
    let quibitous = Starter::new().start().unwrap();
    let rest_uri = quibitous.rest_uri();
    let block_id = qcli.rest().v0().tip(&rest_uri);
    qcli.rest().v0().block().get(block_id, rest_uri);
}

#[test]
pub fn test_correct_error_is_returned_for_incorrect_block_id() {
    let qcli: QCli = Default::default();
    let incorrect_block_id = "e1049ea45726f0b1fc473af54f706546b3331765abf89ae9e6a8333e49621641aa";
    let quibitous = Starter::new().start().unwrap();

    qcli.rest().v0().block().get_expect_fail(
        incorrect_block_id,
        quibitous.rest_uri(),
        "node rejected request because of invalid parameters",
    );
}

#[test]
pub fn test_correct_error_is_returned_for_incorrect_block_id_in_next_block_id_request() {
    let qcli: QCli = Default::default();
    let incorrect_block_id = "e1049ea45726f0b1fc473af54f706546b3331765abf89ae9e6a8333e49621641aa";

    let quibitous = Starter::new().start().unwrap();

    qcli.rest().v0().block().next_expect_fail(
        incorrect_block_id,
        1,
        quibitous.rest_uri(),
        "node rejected request because of invalid parameters",
    );
}
