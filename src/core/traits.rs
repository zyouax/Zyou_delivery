use async_trait::async_trait;
use crate::models::{Parcel, Rate, ShippingLabel, TrackingInfo};
use crate::errors::DeliveryError;

/// Trait pour l'obtention des tarifs d'envoi
#[async_trait]
pub trait RateProvider: Send + Sync {
    /// Obtient les tarifs disponibles pour un colis
    async fn get_rates(&self, parcel: &Parcel) -> Result<Vec<Rate>, DeliveryError>;

    /// Version synchrone (bloquante) de get_rates
    fn get_rates_blocking(&self, parcel: &Parcel) -> Result<Vec<Rate>, DeliveryError>;
}

/// Trait pour la génération d'étiquettes d'expédition
#[async_trait]
pub trait LabelGenerator: Send + Sync {
    /// Génère une étiquette d'expédition pour un colis avec un tarif sélectionné
    async fn generate_label(&self, parcel: &Parcel, rate: &Rate) -> Result<ShippingLabel, DeliveryError>;

    /// Version synchrone (bloquante) de generate_label
    fn generate_label_blocking(&self, parcel: &Parcel, rate: &Rate) -> Result<ShippingLabel, DeliveryError>;
}

/// Trait pour le suivi de l'acheminement des colis
#[async_trait]
pub trait ShipmentTracker: Send + Sync {
    /// Récupère les informations de suivi d'un colis à partir de son numéro de suivi
    async fn track_parcel(&self, tracking_number: &str) -> Result<TrackingInfo, DeliveryError>;

    /// Version synchrone (bloquante) de track_parcel
    fn track_parcel_blocking(&self, tracking_number: &str) -> Result<TrackingInfo, DeliveryError>;

    /// Vérifie si le transporteur peut suivre ce numéro de suivi (basé sur le format)
    fn can_track(&self, tracking_number: &str) -> bool;
}

/// Trait pour la normalisation des données hétérogènes entre transporteurs
pub trait DataNormalizer: Send + Sync {
    /// Convertit un code d'état spécifique au transporteur en un format standardisé
    fn normalize_status_code(&self, carrier_status: &str) -> String;

    /// Normalise une adresse selon les standards du transporteur
    fn normalize_address(&self, address: &mut crate::models::Address) -> Result<(), DeliveryError>;

    /// Vérifie si une adresse est valide pour ce transporteur
    fn validate_address(&self, address: &crate::models::Address) -> Result<(), DeliveryError>;
}

/// Trait combiné pour un transporteur complet
pub trait ShippingCarrier: RateProvider + LabelGenerator + ShipmentTracker + DataNormalizer {
    /// Obtient le code du transporteur
    fn carrier_code(&self) -> crate::models::CarrierCode;

    /// Obtient le nom du transporteur
    fn carrier_name(&self) -> String;

    /// Vérifie si le transporteur est disponible (API accessible, etc.)
    async fn is_available(&self) -> bool;

    /// Version synchrone (bloquante) de is_available
    fn is_available_blocking(&self) -> bool;
}
