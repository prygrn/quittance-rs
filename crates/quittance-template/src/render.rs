use french_amount_words::euro_amount_to_words;
use minijinja::{AutoEscape, Environment, UndefinedBehavior, Value, context};
use quittance_core::Receipt;

use crate::catalog::find_template;
use crate::french_format::{formatted_date, formatted_money};
use crate::signature::is_png_data_uri;
use crate::{IssueDetails, TemplateError};

/// Rend la quittance avec le template demandé.
///
/// `signature_data_uri` est l'image de signature déjà encodée en data URI PNG base64 ;
/// sans elle, la zone de signature reste vide.
///
/// # Errors
///
/// Voir [`TemplateError`].
// Temporaire : `issue` reste inutilisé le temps de l'étape test-first.
#[allow(unused_variables)]
pub fn render_html(
    template_id: &str,
    receipt: &Receipt,
    issue: &IssueDetails,
    signature_data_uri: Option<&str>,
) -> Result<String, TemplateError> {
    let template = find_template(template_id)
        .ok_or_else(|| TemplateError::UnknownTemplate(template_id.to_owned()))?;
    if signature_data_uri.is_some_and(|uri| !is_png_data_uri(uri)) {
        return Err(TemplateError::InvalidSignature);
    }
    let total_in_words = euro_amount_to_words(receipt.total().cents())?;

    let mut environment = Environment::new();
    // Échappement HTML forcé quel que soit le nom du template : toutes les saisies sont hostiles.
    environment.set_auto_escape_callback(|_| AutoEscape::Html);
    environment.set_undefined_behavior(UndefinedBehavior::Strict);
    let landlord = receipt.landlord();
    let tenant = receipt.tenant();
    let html = environment
        .template_from_str(template.source)
        .map_err(rendering_failure)?
        .render(context! {
            landlord_name => landlord.name(),
            landlord_address => landlord.address(),
            landlord_email => landlord.email(),
            tenant_name => tenant.name(),
            tenant_address => tenant.address(),
            tenant_email => tenant.email(),
            property_address => receipt.property_address(),
            period_start => generated_text(formatted_date(receipt.period().start())),
            period_end => generated_text(formatted_date(receipt.period().end())),
            payment_date => generated_text(formatted_date(receipt.payment_date())),
            rent => generated_text(formatted_money(receipt.rent())),
            charges => generated_text(formatted_money(receipt.charges())),
            total => generated_text(formatted_money(receipt.total())),
            total_in_words,
            signature_data_uri,
        })
        .map_err(rendering_failure)?;
    Ok(html)
}

/// Chaînes produites par ce crate à partir de nombres et de dates uniquement : sans
/// balisage possible, elles sont insérées sans échappement. Aucune donnée saisie ne doit
/// transiter par cette fonction.
fn generated_text(text: String) -> Value {
    Value::from_safe_string(text)
}

fn rendering_failure(error: minijinja::Error) -> TemplateError {
    TemplateError::Rendering(Box::new(error))
}
