use quibitous_automation::qcli::JCli;

#[test]
pub fn test_key_to_public() {
    let qcli: JCli = Default::default();
    let private_key = "ed25519_sk1357nu8uaxvdekg6uhqmdd0zcd3tjv3qq0p2029uk6pvfxuks5rzstp5ceq";
    let public_key = qcli.key().convert_to_public_string(private_key.to_owned());
    assert_ne!(public_key, "", "generated key is empty");
}

#[test]
pub fn test_key_to_public_invalid_key() {
    let qcli: JCli = Default::default();
    qcli.key().convert_to_public_string_expect_fail(
        "ed2551ssss9_sk1357nu8uaxvdekg6uhqmdd0zcd3tjv3qq0p2029uk6pvfxuks5rzstp5ceq",
        "invalid checksum",
    );
}

#[test]
pub fn test_key_to_public_invalid_chars_key() {
    let qcli: JCli = Default::default();
    qcli.key().convert_to_public_string_expect_fail(
        "node:: ed2551ssss9_sk1357nu8uaxvdekg6uhqmdd0zcd3tjv3qq0p2029uk6pvfxuks5rzstp5ceq",
        "invalid character",
    );
}

#[test]
pub fn test_private_key_to_public_key() {
    let qcli: JCli = Default::default();
    let private_key = qcli.key().generate("Ed25519Extended");
    let public_key = qcli.key().convert_to_public_string(&private_key);
    assert_ne!(public_key, "", "generated key is empty");
}
