// Modules publics
pub mod core;
pub mod carriers;
pub mod models;
pub mod errors;
pub mod utils;

// Réexportations principales pour faciliter l'utilisation
pub use crate::core::ShippingManager;
pub use crate::errors::DeliveryError;
pub use crate::models::{
    Address, Carrier, CarrierCode, Parcel, Rate,
    ShippingLabel, TrackingEvent, TrackingInfo,
};

// Re-export des traits principaux
pub use crate::core::traits::{
    RateProvider, LabelGenerator,
    ShipmentTracker, DataNormalizer,
};
