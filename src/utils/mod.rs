pub mod validation;
pub mod formatting;
pub mod geo;

/// Utilitaires généraux
pub mod general {
    use crate::models::CarrierCode;

    /// Détecte le transporteur probable à partir d'un numéro de suivi
    pub fn detect_carrier_from_tracking(tracking: &str) -> Option<CarrierCode> {
        // Exemples de formats de numéros de suivi (simplifiés)
        if tracking.len() == 13 && tracking.starts_with("1Z") {
            return Some(CarrierCode::UPS);
        }

        if tracking.len() == 12 && tracking.chars().all(|c| c.is_numeric()) {
            return Some(CarrierCode::FedEx);
        }

        if tracking.len() == 13 && tracking.ends_with("FR") {
            return Some(CarrierCode::Colissimo);
        }

        if tracking.len() == 10 && tracking.starts_with("CH") {
            return Some(CarrierCode::Chronopost);
        }

        None
    }
}

/// Module pour la journalisation
pub mod logging {
    use log::{debug, error, info, warn};

    /// Logs une action d'API avec des détails d'usage
    pub fn log_api_call(carrier: &str, endpoint: &str, status_code: u16, duration_ms: u64) {
        if status_code >= 200 && status_code < 300 {
            info!("API {} - {} - Status: {} - Duration: {}ms", carrier, endpoint, status_code, duration_ms);
        } else {
            warn!("API {} - {} - Status: {} - Duration: {}ms", carrier, endpoint, status_code, duration_ms);
        }
    }

    /// Logs une erreur d'API
    pub fn log_api_error(carrier: &str, endpoint: &str, error: &str) {
        error!("API Error - {} - {}: {}", carrier, endpoint, error);
    }

    /// Logs un événement de suivi
    pub fn log_tracking_event(carrier: &str, tracking: &str, status: &str) {
        debug!("Tracking Event - {} - {} - Status: {}", carrier, tracking, status);
    }
}
