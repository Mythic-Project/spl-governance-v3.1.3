use {
    lazy_static::lazy_static,
    solana_program_test::find_file,
    std::{process::Command, sync::Mutex},
};

lazy_static! {
    pub static ref VOTER_WEIGHT_ADDIN_BUILD_GUARD: Mutex::<u8> = Mutex::new(0);
}
lazy_static! {
    pub static ref SPL_TRANSFER_HOOK_EXAMPLE_BUILD: Mutex::<u8> = Mutex::new(0);
}

pub fn ensure_addin_mock_is_built() {
    if find_file("spl_governance_voter_weight_addin_mock.so").is_none() {
        let _guard = VOTER_WEIGHT_ADDIN_BUILD_GUARD.lock().unwrap();
        if find_file("spl_governance_addin_mock.so").is_none() {
            assert!(Command::new("cargo")
                .args([
                    "build-sbf",
                    "--manifest-path",
                    "../addin-mock/program/Cargo.toml",
                ])
                .status()
                .expect("Failed to build spl-governance-addin-mock program")
                .success());
        }
    }
}

pub fn ensure_transfer_hook_example_is_built() {
    if find_file("spl-transfer-hook-example.so").is_none() {
        let _spl_transfer_hook_example = SPL_TRANSFER_HOOK_EXAMPLE_BUILD.lock().unwrap();
        if find_file("spl-transfer-hook-example.so").is_none() {
            assert!(Command::new("cargo")
                .args([
                    "build-sbf",
                    "--no-default-features",
                    "--manifest-path",
                    "../../token/transfer-hook/example/Cargo.toml",
                ])
                .status()
                .expect("Failed to build spl-transfer-hook-example program")
                .success());
        }
    }
}