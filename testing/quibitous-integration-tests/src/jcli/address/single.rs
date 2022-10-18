use chain_addr::Discrimination;
use quibitous_automation::qcli::QCli;

#[test]
pub fn test_utxo_address_made_of_ed25519_extended_key() {
    let qcli: QCli = Default::default();

    let private_key = qcli.key().generate("ed25519Extended");
    println!("private key: {}", &private_key);

    let public_key = qcli.key().convert_to_public_string(&private_key);
    println!("public key: {}", &public_key);

    let utxo_address = qcli
        .address()
        .single(&public_key, None, Discrimination::Test);
    assert_ne!(utxo_address, "", "generated utxo address is empty");
}

#[test]
pub fn test_delegation_address_made_of_ed25519_extended_seed_key() {
    let qcli: QCli = Default::default();

    let correct_seed = "73855612722627931e20c850f8ad53eb04c615c7601a95747be073dcada3e135";

    let private_key = qcli
        .key()
        .generate_with_seed("ed25519Extended", correct_seed);
    println!("private key: {}", &private_key);

    let public_key = qcli.key().convert_to_public_string(&private_key);
    println!("public key: {}", &public_key);

    let private_key = qcli
        .key()
        .generate_with_seed("ed25519Extended", correct_seed);
    println!("private delegation key: {}", &private_key);
    let delegation_key = qcli.key().convert_to_public_string(&private_key);
    println!("delegation key: {}", &delegation_key);

    let delegation_address =
        qcli.address()
            .delegation(&public_key, &delegation_key, Discrimination::Test);
    assert_ne!(
        delegation_address, "",
        "generated delegation adress is empty"
    );
}

#[test]
pub fn test_delegation_address_is_the_same_as_public() {
    let qcli: QCli = Default::default();

    let correct_seed = "73855612722627931e20c850f8ad53eb04c615c7601a95747be073dcada3e135";

    let private_key = qcli
        .key()
        .generate_with_seed("ed25519Extended", correct_seed);
    println!("private key: {}", &private_key);

    let public_key = qcli.key().convert_to_public_string(&private_key);
    println!("public key: {}", &public_key);

    let delegation_address =
        qcli.address()
            .delegation(&public_key, &public_key, Discrimination::Test);
    assert_ne!(
        delegation_address, "",
        "generated delegation address is empty"
    );
}

#[test]
pub fn test_delegation_address_for_prod_discrimination() {
    let qcli: QCli = Default::default();

    let correct_seed = "73855612722627931e20c850f8ad53eb04c615c7601a95747be073dcada3e135";

    let private_key = qcli
        .key()
        .generate_with_seed("ed25519Extended", correct_seed);
    println!("private key: {}", &private_key);

    let public_key = qcli.key().convert_to_public_string(&private_key);
    println!("public key: {}", &public_key);

    let delegation_address =
        qcli.address()
            .delegation(&public_key, &public_key, Discrimination::Production);
    assert_ne!(
        delegation_address, "",
        "generated delegation address is empty"
    );
}

#[test]
pub fn test_single_address_for_prod_discrimination() {
    let qcli: QCli = Default::default();

    let correct_seed = "73855612722627931e20c850f8ad53eb04c615c7601a95747be073dcada3e135";

    let private_key = qcli
        .key()
        .generate_with_seed("ed25519Extended", correct_seed);
    println!("private key: {}", &private_key);

    let public_key = qcli.key().convert_to_public_string(&private_key);
    println!("public key: {}", &public_key);

    let delegation_address =
        qcli.address()
            .delegation(&public_key, &public_key, Discrimination::Production);
    assert_ne!(delegation_address, "", "generated single address is empty");
}

#[test]
pub fn test_account_address_for_prod_discrimination() {
    let qcli: QCli = Default::default();

    let correct_seed = "73855612722627931e20c850f8ad53eb04c615c7601a95747be073dcada3e135";

    let private_key = qcli
        .key()
        .generate_with_seed("ed25519Extended", correct_seed);
    println!("private key: {}", &private_key);

    let public_key = qcli.key().convert_to_public_string(&private_key);
    println!("public key: {}", &public_key);

    let delegation_address =
        qcli.address()
            .delegation(&public_key, &public_key, Discrimination::Production);
    assert_ne!(delegation_address, "", "generated account address is empty");
}
#[test]
pub fn test_utxo_address_made_of_incorrect_ed25519_extended_key() {
    let qcli: QCli = Default::default();

    let private_key = qcli.key().generate("ed25519Extended");
    println!("private key: {}", &private_key);

    let mut public_key = qcli.key().convert_to_public_string(&private_key);
    println!("public key: {}", &public_key);

    public_key.push('A');

    // Assertion changed due to issue #306. After fix please change it to correct one
    qcli.address().single_expect_fail(
        &public_key,
        None,
        Discrimination::Test,
        "Failed to parse bech32, invalid data format",
    );
}

#[test]
pub fn test_delegation_address_made_of_random_string() {
    let qcli: QCli = Default::default();

    let private_key = qcli.key().generate("ed25519Extended");
    println!("private key: {}", &private_key);

    let public_key = qcli.key().convert_to_public_string(&private_key);
    println!("public key: {}", &public_key);

    let delegation_key = "adfasdfasdfdasfasdfadfasdf";

    // Assertion changed due to issue #306. After fix please change it to correct one
    qcli.address().delegation_expect_fail(
        public_key,
        delegation_key,
        Discrimination::Test,
        "Failed to parse bech32, invalid data format",
    );
}

#[test]
pub fn test_delegation_address_made_of_incorrect_public_ed25519_extended_key() {
    let qcli: QCli = Default::default();

    let private_key = qcli.key().generate("ed25519Extended");
    println!("private key: {}", &private_key);

    let mut public_key = qcli.key().convert_to_public_string(&private_key);
    println!("public key: {}", &public_key);

    let private_key = qcli.key().generate("ed25519Extended");
    println!("private delegation key: {}", &private_key);
    let delegation_key = qcli.key().convert_to_public_string(&private_key);
    println!("delegation key: {}", &delegation_key);

    public_key.push('A');

    // Assertion changed due to issue #306. After fix please change it to correct one
    qcli.address().delegation_expect_fail(
        public_key,
        delegation_key,
        Discrimination::Test,
        "Failed to parse bech32, invalid data format",
    );
}
