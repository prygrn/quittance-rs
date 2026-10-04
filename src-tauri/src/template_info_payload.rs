use quittance_template::TemplateInfo;
use serde::Serialize;

/// Modèle de quittance tel que l'UI le reçoit (`TemplateInfo` de `src/api.ts`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TemplateInfoPayload {
    pub id: &'static str,
    pub label: &'static str,
}

impl From<TemplateInfo> for TemplateInfoPayload {
    fn from(template: TemplateInfo) -> Self {
        Self {
            id: template.id,
            label: template.label,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;
    use crate::test_support::{from_ipc_json, to_ipc_json};

    /// Forme du modèle vue par l'UI.
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct UiTemplateInfo {
        id: String,
        label: String,
    }

    #[test]
    fn given_template_info_when_serializing_then_ui_receives_id_and_label() {
        let payload = TemplateInfoPayload::from(TemplateInfo {
            id: "standard",
            label: "Quittance standard",
        });

        let ui_template: UiTemplateInfo = from_ipc_json(&to_ipc_json(payload)).unwrap();

        assert_eq!(
            ui_template,
            UiTemplateInfo {
                id: "standard".to_owned(),
                label: "Quittance standard".to_owned(),
            }
        );
    }
}
