pub mod traits;

use std::collections::HashMap;
use std::sync::Arc;

use crate::errors::DeliveryError;
use crate::models::{CarrierCode, Parcel, Rate, ShippingLabel, TrackingInfo};
use traits::ShippingCarrier;

/// Le gestionnaire principal pour interagir avec différents transporteurs
pub struct ShippingManager {
    carriers: HashMap<CarrierCode, Arc<dyn ShippingCarrier>>,
}

impl ShippingManager {
    /// Crée un nouveau gestionnaire de livraison vide
    pub fn new() -> Self {
        Self {
            carriers: HashMap::new(),
        }
    }

    /// Ajoute un transporteur au gestionnaire
    pub fn add_carrier(&mut self, carrier: Box<dyn ShippingCarrier>) -> &mut Self {
        let code = carrier.carrier_code();
        self.carriers.insert(code, Arc::from(carrier));
        self
    }

    /// Obtient un transporteur par son code
    pub fn get_carrier(&self, code: &CarrierCode) -> Option<Arc<dyn ShippingCarrier>> {
        self.carriers.get(code).cloned()
    }

    /// Liste tous les transporteurs disponibles
    pub fn list_carriers(&self) -> Vec<(CarrierCode, String)> {
        self.carriers
            .iter()
            .map(|(code, carrier)| (*code, carrier.carrier_name()))
            .collect()
    }

    /// Obtient les tarifs de tous les transporteurs pour un colis donné
    pub async fn get_all_rates(&self, parcel: &Parcel) -> HashMap<CarrierCode, Result<Vec<Rate>, DeliveryError>> {
        let mut results = HashMap::new();

        for (code, carrier) in &self.carriers {
            let result = carrier.get_rates(parcel).await;
            results.insert(*code, result);
        }

        results
    }

    /// Obtient les tarifs d'un transporteur spécifique pour un colis donné
    pub async fn get_rates(&self, carrier_code: &CarrierCode, parcel: &Parcel) -> Result<Vec<Rate>, DeliveryError> {
        let carrier = self.get_carrier(carrier_code)
            .ok_or_else(|| DeliveryError::UnknownCarrier(format!("{:?}", carrier_code)))?;

        carrier.get_rates(parcel).await
    }

    /// Version synchrone de get_rates
    pub fn get_rates_blocking(&self, carrier_code: &CarrierCode, parcel: &Parcel) -> Result<Vec<Rate>, DeliveryError> {
        let carrier = self.get_carrier(carrier_code)
            .ok_or_else(|| DeliveryError::UnknownCarrier(format!("{:?}", carrier_code)))?;

        carrier.get_rates_blocking(parcel)
    }

    /// Génère une étiquette d'expédition pour un colis avec un tarif sélectionné
    pub async fn generate_label(
        &self,
        carrier_code: &CarrierCode,
        parcel: &Parcel,
        rate: &Rate
    ) -> Result<ShippingLabel, DeliveryError> {
        let carrier = self.get_carrier(carrier_code)
            .ok_or_else(|| DeliveryError::UnknownCarrier(format!("{:?}", carrier_code)))?;

        carrier.generate_label(parcel, rate).await
    }

    /// Version synchrone de generate_label
    pub fn generate_label_blocking(
        &self,
        carrier_code: &CarrierCode,
        parcel: &Parcel,
        rate: &Rate
    ) -> Result<ShippingLabel, DeliveryError> {
        let carrier = self.get_carrier(carrier_code)
            .ok_or_else(|| DeliveryError::UnknownCarrier(format!("{:?}", carrier_code)))?;

        carrier.generate_label_blocking(parcel, rate)
    }

    /// Suit un colis à partir de son numéro de suivi
    /// Essaie de détecter automatiquement le transporteur approprié
    pub async fn track_parcel(&self, tracking_number: &str) -> Result<TrackingInfo, DeliveryError> {
        // Cherche le transporteur capable de suivre ce numéro
        for carrier in self.carriers.values() {
            if carrier.can_track(tracking_number) {
                return carrier.track_parcel(tracking_number).await;
            }
        }

        Err(DeliveryError::UnsupportedTrackingNumber(tracking_number.to_string()))
    }

    /// Version synchrone de track_parcel
    pub fn track_parcel_blocking(&self, tracking_number: &str) -> Result<TrackingInfo, DeliveryError> {
        // Cherche le transporteur capable de suivre ce numéro
        for carrier in self.carriers.values() {
            if carrier.can_track(tracking_number) {
                return carrier.track_parcel_blocking(tracking_number);
            }
        }

        Err(DeliveryError::UnsupportedTrackingNumber(tracking_number.to_string()))
    }

    /// Vérifie si un transporteur spécifique est disponible
    pub async fn is_carrier_available(&self, carrier_code: &CarrierCode) -> bool {
        match self.get_carrier(carrier_code) {
            Some(carrier) => carrier.is_available().await,
            None => false,
        }
    }
}

impl Default for ShippingManager {
    fn default() -> Self {
        Self::new()
    }
}
