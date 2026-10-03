use quittance_core::Date;

/// Mention « Fait à …, le … » de la quittance, fournie par l'appelant : le lieu vient de
/// la configuration du bailleur, la date est celle de la génération. Le crate ne lit pas
/// l'horloge, pour que le rendu reste une fonction pure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueDetails {
    /// Lieu d'émission, saisi par l'utilisateur : il est échappé au rendu.
    pub place: String,
    /// Date d'émission.
    pub date: Date,
}
