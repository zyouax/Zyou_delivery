pub mod types;
pub mod carriers;
pub mod error;

use types::{Carrier, Shipment, ShippingRate, LabelResponse, TrackingResponse};
use error::ShippingError;

pub fn init() {
    dotenv::dotenv().ok();
    if !cfg!(any(feature = "colissimo", feature = "fedex", feature = "ups", feature = "tnt", feature = "chronopost", feature = "mondialrelay")) {
        log::warn!("No carrier features enabled. All operations will fail.");
    }
}

pub async fn get_shipping_rates(carrier: Carrier, shipment: &Shipment) -> Result<Vec<ShippingRate>, ShippingError> {
    match carrier {
        #[cfg(feature = "colissimo")]
        Carrier::Colissimo => carriers::colissimo::get_rates(shipment).await,
        #[cfg(feature = "fedex")]
        Carrier::FedEx => carriers::fedex::get_rates(shipment).await,
        #[cfg(feature = "chronopost")]
        Carrier::Chronopost => carriers::chronopost::get_rates(shipment).await,
        #[cfg(feature = "tnt")]
        Carrier::TNT => carriers::tnt::get_rates(shipment).await,
        #[cfg(feature = "ups")]
        Carrier::UPS => carriers::ups::get_rates(shipment).await,
        #[cfg(feature = "mondialrelay")]
        Carrier::MondialRelay => carriers::mondialrelay::get_rates(shipment).await,
        _ => Err(ShippingError::NotImplemented(format!("Carrier {:?}", carrier))),
    }
}

pub async fn create_shipping_label(carrier: Carrier, shipment: &Shipment) -> Result<LabelResponse, ShippingError> {
    match carrier {
        #[cfg(feature = "colissimo")]
        Carrier::Colissimo => carriers::colissimo::create_label(shipment).await,
        #[cfg(feature = "fedex")]
        Carrier::FedEx => carriers::fedex::create_label(shipment).await,
        #[cfg(feature = "chronopost")]
        Carrier::Chronopost => carriers::chronopost::create_label(shipment).await,
        #[cfg(feature = "tnt")]
        Carrier::TNT => carriers::tnt::create_label(shipment).await,
        #[cfg(feature = "ups")]
        Carrier::UPS => carriers::ups::create_label(shipment).await,
        #[cfg(feature = "mondialrelay")]
        Carrier::MondialRelay => carriers::mondialrelay::create_label(shipment).await,
        _ => Err(ShippingError::NotImplemented(format!("Carrier {:?}", carrier))),
    }
}

pub async fn track_package(carrier: Carrier, tracking_number: &str) -> Result<TrackingResponse, ShippingError> {
    match carrier {
        #[cfg(feature = "colissimo")]
        Carrier::Colissimo => carriers::colissimo::track_package(tracking_number).await,
        #[cfg(feature = "fedex")]
        Carrier::FedEx => carriers::fedex::track_package(tracking_number).await,
        #[cfg(feature = "chronopost")]
        Carrier::Chronopost => carriers::chronopost::track_package(tracking_number).await,
        #[cfg(feature = "tnt")]
        Carrier::TNT => carriers::tnt::track_package(tracking_number).await,
        #[cfg(feature = "ups")]
        Carrier::UPS => carriers::ups::track_package(tracking_number).await,
        #[cfg(feature = "mondialrelay")]
        Carrier::MondialRelay => carriers::mondialrelay::track_package(tracking_number).await,
        _ => Err(ShippingError::NotImplemented(format!("Carrier {:?}", carrier))),
    }
}