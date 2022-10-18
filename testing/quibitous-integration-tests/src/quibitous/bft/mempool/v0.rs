use assert_fs::TempDir;
use quibitous_automation::quibitous::{ConfigurationBuilder, Starter};
use quibitous_automation::testing::time;
use quibitous_lib::interfaces::BlockDate;
use quibitous_lib::interfaces::InitialUTxO;
use quibitous_lib::interfaces::Mempool;
use std::time::Duration;
use thor::{FragmentSender, FragmentSenderSetup, FragmentVerifier, VerifyExitStrategy};

#[test]
pub fn test_mempool_pool_max_entries_limit() {
    let temp_dir = TempDir::new().unwrap();

    let receiver = thor::Wallet::default();
    let mut sender = thor::Wallet::default();

    let leader_config = ConfigurationBuilder::new()
        .with_funds(vec![
            InitialUTxO {
                address: sender.address(),
                value: 100.into(),
            },
            InitialUTxO {
                address: receiver.address(),
                value: 100.into(),
            },
        ])
        // Use a long slot time to avoid producing a block
        // before both test requests has been sent
        .with_slot_duration(15)
        .with_mempool(Mempool {
            pool_max_entries: 1.into(),
            log_max_entries: 100.into(),
            persistent_log: None,
        })
        .build(&temp_dir);

    let quibitous = Starter::new()
        .config(leader_config)
        .temp_dir(temp_dir)
        .start()
        .unwrap();

    let fragment_sender = FragmentSender::from_with_setup(
        quibitous.block0_configuration(),
        FragmentSenderSetup::no_verify(),
    );

    let verifier = quibitous
        .correct_state_verifier()
        .record_address_state(vec![&sender.address(), &receiver.address()]);

    let mempool_check = fragment_sender
        .send_transaction(&mut sender, &receiver, &quibitous, 1.into())
        .unwrap();

    fragment_sender
        .send_transaction(&mut sender, &receiver, &quibitous, 1.into())
        .unwrap();

    // Wait until the fragment enters the mempool
    FragmentVerifier::wait_fragment(
        Duration::from_millis(100),
        mempool_check.clone(),
        VerifyExitStrategy::OnPending,
        &quibitous,
    )
    .unwrap();

    quibitous
        .correct_state_verifier()
        .fragment_logs()
        .assert_size(1)
        .assert_contains_only(mempool_check.fragment_id());

    FragmentVerifier::wait_and_verify_is_in_block(
        Duration::from_secs(2),
        mempool_check,
        &quibitous,
    )
    .unwrap();

    verifier
        .value_moved_between_addresses(&sender.address(), &receiver.address(), 1.into())
        .unwrap();
}

#[test]
pub fn test_mempool_pool_max_entries_equal_0() {
    let temp_dir = TempDir::new().unwrap();

    let receiver = thor::Wallet::default();
    let mut sender = thor::Wallet::default();

    let config = ConfigurationBuilder::new()
        .with_funds(vec![
            InitialUTxO {
                address: sender.address(),
                value: 100.into(),
            },
            InitialUTxO {
                address: receiver.address(),
                value: 100.into(),
            },
        ])
        .with_mempool(Mempool {
            pool_max_entries: 0.into(),
            log_max_entries: 100.into(),
            persistent_log: None,
        })
        .build(&temp_dir);

    let quibitous = Starter::new()
        .config(config)
        .temp_dir(temp_dir)
        .start()
        .unwrap();

    let verifier = quibitous
        .correct_state_verifier()
        .record_address_state(vec![&sender.address(), &receiver.address()]);

    let fragment_sender = FragmentSender::from_with_setup(
        quibitous.block0_configuration(),
        FragmentSenderSetup::no_verify(),
    );

    fragment_sender
        .send_transaction(&mut sender, &receiver, &quibitous, 1.into())
        .unwrap();

    fragment_sender
        .send_transaction(&mut sender, &receiver, &quibitous, 1.into())
        .unwrap();

    quibitous
        .correct_state_verifier()
        .fragment_logs()
        .assert_empty();

    time::wait_for_date(BlockDate::new(0, 5), quibitous.rest());
    verifier
        .no_changes(vec![&sender.address(), &receiver.address()])
        .unwrap();
}

#[test]
pub fn test_mempool_log_max_entries_only_one_fragment() {
    let temp_dir = TempDir::new().unwrap();

    let receiver = thor::Wallet::default();
    let mut sender = thor::Wallet::default();

    let config = ConfigurationBuilder::new()
        .with_funds(vec![
            InitialUTxO {
                address: sender.address(),
                value: 100.into(),
            },
            InitialUTxO {
                address: receiver.address(),
                value: 100.into(),
            },
        ])
        // Use a long slot time to avoid producing a block
        // before both test requests has been sent
        .with_slot_duration(15)
        .with_mempool(Mempool {
            pool_max_entries: 1.into(),
            log_max_entries: 1.into(),
            persistent_log: None,
        })
        .build(&temp_dir);

    let quibitous = Starter::new()
        .config(config)
        .temp_dir(temp_dir)
        .start()
        .unwrap();

    let verifier = quibitous
        .correct_state_verifier()
        .record_address_state(vec![&sender.address(), &receiver.address()]);

    let fragment_sender = FragmentSender::from_with_setup(
        quibitous.block0_configuration(),
        FragmentSenderSetup::no_verify(),
    );

    let first_fragment = fragment_sender
        .send_transaction(&mut sender, &receiver, &quibitous, 1.into())
        .unwrap();

    let _second_fragment = fragment_sender
        .send_transaction(&mut sender, &receiver, &quibitous, 1.into())
        .unwrap();

    // Wait until the fragment enters the mempool
    FragmentVerifier::wait_fragment(
        Duration::from_millis(100),
        first_fragment.clone(),
        VerifyExitStrategy::OnPending,
        &quibitous,
    )
    .unwrap();

    quibitous
        .correct_state_verifier()
        .fragment_logs()
        .assert_size(1)
        .assert_contains_only(first_fragment.fragment_id());

    FragmentVerifier::wait_and_verify_is_in_block(
        Duration::from_secs(15),
        first_fragment,
        &quibitous,
    )
    .unwrap();

    verifier
        .value_moved_between_addresses(&sender.address(), &receiver.address(), 1.into())
        .unwrap();
}

#[test]
pub fn test_mempool_log_max_entries_equals_0() {
    let temp_dir = TempDir::new().unwrap();

    let receiver = thor::Wallet::default();
    let mut sender = thor::Wallet::default();

    let config = ConfigurationBuilder::new()
        .with_funds(vec![
            InitialUTxO {
                address: sender.address(),
                value: 100.into(),
            },
            InitialUTxO {
                address: receiver.address(),
                value: 100.into(),
            },
        ])
        .with_mempool(Mempool {
            pool_max_entries: 0.into(),
            log_max_entries: 0.into(),
            persistent_log: None,
        })
        .build(&temp_dir);

    let quibitous = Starter::new()
        .config(config)
        .temp_dir(temp_dir)
        .start()
        .unwrap();

    let verifier = quibitous
        .correct_state_verifier()
        .record_address_state(vec![&sender.address(), &receiver.address()]);

    let fragment_sender = FragmentSender::from_with_setup(
        quibitous.block0_configuration(),
        FragmentSenderSetup::no_verify(),
    );

    fragment_sender
        .send_transaction(&mut sender, &receiver, &quibitous, 1.into())
        .unwrap();

    fragment_sender
        .send_transaction(&mut sender, &receiver, &quibitous, 1.into())
        .unwrap();

    quibitous
        .correct_state_verifier()
        .fragment_logs()
        .assert_empty();

    time::wait_for_date(BlockDate::new(0, 5), quibitous.rest());

    verifier
        .no_changes(vec![&sender.address(), &receiver.address()])
        .unwrap();
}

#[test]
pub fn test_mempool_pool_max_entries_overrides_log_max_entries() {
    let temp_dir = TempDir::new().unwrap();

    let receiver = thor::Wallet::default();
    let mut sender = thor::Wallet::default();

    let config = ConfigurationBuilder::new()
        .with_funds(vec![
            InitialUTxO {
                address: sender.address(),
                value: 100.into(),
            },
            InitialUTxO {
                address: receiver.address(),
                value: 100.into(),
            },
        ])
        .with_mempool(Mempool {
            pool_max_entries: 2.into(),
            log_max_entries: 0.into(),
            persistent_log: None,
        })
        .build(&temp_dir);

    let quibitous = Starter::new()
        .config(config)
        .temp_dir(temp_dir)
        .start()
        .unwrap();

    let verifier = quibitous
        .correct_state_verifier()
        .record_address_state(vec![&sender.address(), &receiver.address()]);

    let fragment_sender = FragmentSender::from(quibitous.block0_configuration());

    fragment_sender
        .send_transaction(&mut sender, &receiver, &quibitous, 1.into())
        .unwrap();

    let second_transaction = fragment_sender
        .send_transaction(&mut sender, &receiver, &quibitous, 1.into())
        .unwrap();

    // Wait until the fragment enters the mempool
    FragmentVerifier::wait_fragment(
        Duration::from_millis(100),
        second_transaction,
        VerifyExitStrategy::OnPending,
        &quibitous,
    )
    .unwrap();

    quibitous
        .correct_state_verifier()
        .fragment_logs()
        .assert_size(2);

    time::wait_for_date(BlockDate::new(0, 10), quibitous.rest());

    verifier
        .value_moved_between_addresses(&sender.address(), &receiver.address(), 2.into())
        .unwrap();
}
