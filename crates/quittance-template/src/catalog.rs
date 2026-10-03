/// Template proposé à l'utilisateur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemplateInfo {
    /// Identifiant stable, passé à [`crate::render_html`].
    pub id: &'static str,
    /// Libellé affiché, en français.
    pub label: &'static str,
}

/// Template embarqué dans le binaire avec sa description.
pub(crate) struct EmbeddedTemplate {
    pub(crate) info: TemplateInfo,
    pub(crate) source: &'static str,
}

const EMBEDDED_TEMPLATES: [EmbeddedTemplate; 1] = [EmbeddedTemplate {
    info: TemplateInfo {
        id: "standard",
        label: "Quittance standard",
    },
    source: include_str!("../templates/standard.html.j2"),
}];

/// Templates disponibles, dans l'ordre de présentation.
pub fn list_templates() -> Vec<TemplateInfo> {
    EMBEDDED_TEMPLATES
        .iter()
        .map(|template| template.info)
        .collect()
}

pub(crate) fn find_template(template_id: &str) -> Option<&'static EmbeddedTemplate> {
    EMBEDDED_TEMPLATES
        .iter()
        .find(|template| template.info.id == template_id)
}
