use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use quibitous_automation::quibitous::{download_last_n_releases, get_quibitous_bin};
use quibitous_automation::quibitous::{ConfigurationBuilder, Starter, StartupVerificationMode};
use quibitous_automation::testing::{BranchCount, StopCriteria, StorageBuilder};
use quibitous_lib::interfaces::ActiveSlotCoefficient;
use std::time::Duration;

#[test]
#[ignore]
pub fn bootstrap_from_100_mb_storage() {
    let storage_size = 100;
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let child = temp_dir.child("storage");
    let path = child.path();
    let storage_builder = StorageBuilder::new(
        BranchCount::Unlimited,
        StopCriteria::SizeInMb(storage_size),
        path,
    );
    storage_builder.build();

    let config = ConfigurationBuilder::new()
        .with_slots_per_epoch(20)
        .with_consensus_genesis_optimum_active_slot_coeff(ActiveSlotCoefficient::MAXIMUM)
        .with_storage(&child)
        .build(&temp_dir);

    let quibitous = Starter::new()
        .timeout(Duration::from_secs(24_000))
        .config(config.clone())
        .benchmark(&format!("bootstrap from {} MB storage", storage_size))
        .verify_by(StartupVerificationMode::Rest)
        .start()
        .unwrap();

    quibitous.shutdown();

    let quibitous = Starter::new()
        .timeout(Duration::from_secs(24_000))
        .config(config.clone())
        .benchmark(&format!(
            "bootstrap from {} MB storage after restart",
            storage_size
        ))
        .verify_by(StartupVerificationMode::Rest)
        .start()
        .unwrap();

    quibitous.stop();

    let _quibitous = Starter::new()
        .timeout(Duration::from_secs(24_000))
        .config(config)
        .benchmark(&format!(
            "bootstrap from {} MB storage after kill",
            storage_size
        ))
        .verify_by(StartupVerificationMode::Rest)
        .start()
        .unwrap();
}

#[test]
#[ignore]
pub fn legacy_bootstrap_from_1_gb_storage() {
    let storage_size = 1_000;
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let child = temp_dir.child("storage");
    let path = child.path();
    let storage_builder = StorageBuilder::new(
        BranchCount::Unlimited,
        StopCriteria::SizeInMb(storage_size),
        path,
    );
    storage_builder.build();

    let config = ConfigurationBuilder::new()
        .with_slots_per_epoch(20)
        .with_consensus_genesis_optimum_active_slot_coeff(ActiveSlotCoefficient::MAXIMUM)
        .with_storage(&child)
        .build(&temp_dir);

    let legacy_release = download_last_n_releases(1).iter().cloned().next().unwrap();
    let quibitous_app = get_quibitous_bin(&legacy_release, &temp_dir);

    let quibitous = Starter::new()
        .timeout(Duration::from_secs(24_000))
        .config(config.clone())
        .legacy(legacy_release.version())
        .quibitous_app(quibitous_app.clone())
        .benchmark(&format!(
            "legacy {} bootstrap from {} MB storage",
            legacy_release.version(),
            storage_size
        ))
        .verify_by(StartupVerificationMode::Rest)
        .start()
        .unwrap();

    quibitous.shutdown();

    let quibitous = Starter::new()
        .timeout(Duration::from_secs(24_000))
        .config(config.clone())
        .legacy(legacy_release.version())
        .quibitous_app(quibitous_app.clone())
        .benchmark(&format!(
            "legacy {} bootstrap from {} MB storage after restart",
            legacy_release.version(),
            storage_size
        ))
        .verify_by(StartupVerificationMode::Rest)
        .start()
        .unwrap();

    quibitous.stop();

    let _quibitous = Starter::new()
        .timeout(Duration::from_secs(24_000))
        .config(config)
        .legacy(legacy_release.version())
        .quibitous_app(quibitous_app)
        .benchmark(&format!(
            "legacy {} bootstrap from {} MB storage after kill",
            legacy_release.version(),
            storage_size
        ))
        .verify_by(StartupVerificationMode::Rest)
        .start()
        .unwrap();
}
