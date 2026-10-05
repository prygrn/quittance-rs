//! Configuration du bundle Tauri, lue comme le fait la CLI.

use std::path::Path;

use tauri::utils::config::{BundleResources, Config};

use crate::test_support::from_ipc_json;

const DEJAVU_LICENSE_SOURCE: &str = "../crates/quittance-signature/assets/DejaVuSans-LICENSE.txt";

fn tauri_config() -> Config {
    from_ipc_json(include_str!("../tauri.conf.json")).unwrap()
}

#[test]
fn given_bundle_config_when_reading_resources_then_dejavu_license_is_shipped() {
    let resources = tauri_config().bundle.resources;

    let Some(BundleResources::Map(resources)) = resources else {
        panic!("bundle resources should map sources to targets, got {resources:?}");
    };
    assert_eq!(
        resources.get(DEJAVU_LICENSE_SOURCE).map(String::as_str),
        Some("licenses/DejaVuSans-LICENSE.txt")
    );
}

#[test]
fn given_dejavu_license_source_when_resolving_from_the_app_crate_then_file_exists() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join(DEJAVU_LICENSE_SOURCE);

    assert!(source.is_file(), "{} should exist", source.display());
}
