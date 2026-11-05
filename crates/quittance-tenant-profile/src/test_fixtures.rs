use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{TenantProfile, TenantProfileFields};

pub(crate) const KNOWN_TEMPLATE_IDS: &[&str] = &["standard"];

/// Fichier de la fiche de [`sample_fields`], au format du README.
pub(crate) const SAMPLE_TOML: &str = r#"template_id = "standard"
tenant_name = "Jeanne Martin"
tenant_address = "8 rue Victor Hugo, 75011 Paris"
tenant_email = "jeanne.martin@example.fr"
property_address = "12 rue des Lilas, 75011 Paris"
rent = "650,00"
charges = "50,50"
"#;

pub(crate) fn sample_fields() -> TenantProfileFields {
    TenantProfileFields {
        template_id: "standard".to_owned(),
        tenant_name: "Jeanne Martin".to_owned(),
        tenant_address: "8 rue Victor Hugo, 75011 Paris".to_owned(),
        tenant_email: "jeanne.martin@example.fr".to_owned(),
        property_address: "12 rue des Lilas, 75011 Paris".to_owned(),
        rent_cents: 65_000,
        charges_cents: 5_050,
    }
}

pub(crate) fn sample_profile() -> TenantProfile {
    TenantProfile::new(sample_fields(), KNOWN_TEMPLATE_IDS).unwrap()
}

pub(crate) fn profile_named(tenant_name: &str) -> TenantProfile {
    let fields = TenantProfileFields {
        tenant_name: tenant_name.to_owned(),
        ..sample_fields()
    };
    TenantProfile::new(fields, KNOWN_TEMPLATE_IDS).unwrap()
}

/// Chemin propre à un test dans le dossier temporaire du système, absent au départ.
pub(crate) fn unique_temp_path(test_name: &str) -> PathBuf {
    let nanoseconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "quittance-tenant-profile-{test_name}-{}-{nanoseconds}",
        process::id()
    ))
}
