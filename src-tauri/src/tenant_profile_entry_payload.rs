use quittance_tenant_profile::TenantProfileEntry;
use serde::Serialize;

/// Fiche proposée dans la liste de l'UI (`TenantProfileEntry` de `src/api.ts`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TenantProfileEntryPayload {
    pub id: String,
    pub label: String,
}

impl From<TenantProfileEntry> for TenantProfileEntryPayload {
    fn from(entry: TenantProfileEntry) -> Self {
        Self {
            id: entry.id,
            label: entry.label,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;
    use crate::test_support::{from_ipc_json, to_ipc_json};

    /// Forme de l'entrée vue par l'UI.
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct UiTenantProfileEntry {
        id: String,
        label: String,
    }

    #[test]
    fn given_entry_when_serializing_then_ui_receives_id_and_label() {
        let payload = TenantProfileEntryPayload::from(TenantProfileEntry {
            id: "jeanne-martin".to_owned(),
            label: "Jeanne Martin".to_owned(),
        });

        let ui_entry: UiTenantProfileEntry = from_ipc_json(&to_ipc_json(payload)).unwrap();

        assert_eq!(
            ui_entry,
            UiTenantProfileEntry {
                id: "jeanne-martin".to_owned(),
                label: "Jeanne Martin".to_owned(),
            }
        );
    }
}
