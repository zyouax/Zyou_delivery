use serde::{Serialize, Deserialize};
use crate::error::ShippingError;

#[derive(Debug, Clone)]
pub enum Carrier {
    Colissimo,
    FedEx,
    Chronopost,
    TNT,
    UPS,
    MondialRelay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub name: String,
    pub street: String,
    pub city: String,
    pub postal_code: String,
    pub country_code: String,
    pub phone: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub weight_kg: f32,
    pub length_cm: f32,
    pub width_cm: f32,
    pub height_cm: f32,
    pub value_eur: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shipment {
    pub sender: Address,
    pub recipient: Address,
    pub package: Package,
}

impl Shipment {
    pub fn validate(&self) -> Result<(), ShippingError> {
        if self.package.weight_kg <= 0.0 {
            return Err(ShippingError::InvalidInput("Le poids doit être positif".to_string()));
        }
        if self.sender.postal_code.is_empty() || self.recipient.postal_code.is_empty() {
            return Err(ShippingError::InvalidInput("Le code postal ne peut pas être vide".to_string()));
        }
        if self.sender.country_code.is_empty() || self.recipient.country_code.is_empty() {
            return Err(ShippingError::InvalidInput("Le code pays ne peut pas être vide".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingRate {
    pub service_name: String,
    pub price_eur: f32,
    pub estimated_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelResponse {
    pub label_url: String,
    pub tracking_number: String,
    pub expiry_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingResponse {
    pub status: String,
    pub last_updated: String,
    pub events: Vec<TrackingEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingEvent {
    pub date: String,
    pub location: String,
    pub description: String,
}