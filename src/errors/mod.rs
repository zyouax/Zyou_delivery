use thiserror::Error;

/// Les erreurs possibles lors de l'utilisation de la bibliothèque
#[derive(Error, Debug)]
pub enum DeliveryError {
    #[error("Erreur d'API du transporteur: {0}")]
    ApiError(String),

    #[error("Transporteur inconnu: {0}")]
    UnknownCarrier(String),

    #[error("Service non supporté: {0}")]
    UnsupportedService(String),

    #[error("Format de numéro de suivi non supporté: {0}")]
    UnsupportedTrackingNumber(String),

    #[error("Colis invalide: {0}")]
    InvalidParcel(String),

    #[error("Adresse invalide: {0}")]
    InvalidAddress(String),

    #[error("Taux non disponible")]
    RateUnavailable,

    #[error("Authentification invalide")]
    AuthenticationError,

    #[error("Erreur lors de la génération de l'étiquette: {0}")]
    LabelGenerationError(String),

    #[error("Erreur de connexion: {0}")]
    ConnectionError(String),

    #[error("Erreur de sérialisation: {0}")]
    SerializationError(String),

    #[error("Opération annulée: {0}")]
    OperationCancelled(String),

    #[error("Erreur interne: {0}")]
    InternalError(String),

    #[error("Erreur HTTP: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Erreur d'E/S: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Erreur de sérialisation JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Erreur inconnue: {0}")]
    Unknown(String),
}

/// Résultat spécifique à la bibliothèque
pub type DeliveryResult<T> = Result<T, DeliveryError>;
