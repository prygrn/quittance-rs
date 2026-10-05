//! Configuration du bundle Tauri, lue comme le fait la CLI.

use std::collections::HashMap;
use std::path::Path;

use tauri::utils::config::{BundleResources, Config, CspDirectiveSources};

use crate::test_support::from_ipc_json;

const DEJAVU_LICENSE_SOURCE: &str = "../crates/quittance-signature/assets/DejaVuSans-LICENSE.txt";

fn tauri_config() -> Config {
    from_ipc_json(include_str!("../tauri.conf.json")).unwrap()
}

/// Sources d'une directive de la CSP de l'app construite ; vide si la directive est absente.
fn csp_sources(directive: &str) -> Vec<String> {
    let csp = tauri_config()
        .app
        .security
        .csp
        .expect("the built app should declare a content security policy");
    let mut directives: HashMap<String, CspDirectiveSources> = csp.into();
    directives
        .remove(directive)
        .map(Vec::from)
        .unwrap_or_default()
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

#[test]
fn given_csp_when_reading_script_src_then_only_app_scripts_are_allowed() {
    let sources = csp_sources("script-src");

    assert!(sources.contains(&"'self'".to_owned()), "{sources:?}");
    assert!(
        !sources.contains(&"'unsafe-inline'".to_owned()),
        "{sources:?}"
    );
    assert!(
        !sources.contains(&"'unsafe-eval'".to_owned()),
        "{sources:?}"
    );
}

#[test]
fn given_csp_when_reading_object_src_then_plugins_are_refused() {
    let sources = csp_sources("object-src");

    assert_eq!(sources, vec!["'none'".to_owned()]);
}

#[test]
fn given_csp_when_reading_connect_src_then_tauri_ipc_is_allowed() {
    let sources = csp_sources("connect-src");

    assert!(sources.contains(&"ipc:".to_owned()), "{sources:?}");
    assert!(
        sources.contains(&"http://ipc.localhost".to_owned()),
        "{sources:?}"
    );
}

#[test]
fn given_csp_when_reading_img_src_then_preview_signature_data_uri_is_allowed() {
    let sources = csp_sources("img-src");

    assert!(sources.contains(&"data:".to_owned()), "{sources:?}");
}

// Un `<style>` dans `index.html` ferait ajouter par Tauri un nonce à `style-src`, ce qui
// désactive `'unsafe-inline'` et bloque les styles de l'aperçu en `iframe srcdoc`.
#[test]
fn given_app_page_when_reading_markup_then_it_holds_no_style_element() {
    let markup = include_str!("../../index.html").to_lowercase();

    assert!(!markup.contains("<style"));
}
