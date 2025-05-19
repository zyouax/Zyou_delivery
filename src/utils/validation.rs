use crate::models::{Address, Parcel};
use crate::errors::DeliveryError;

/// Valide une adresse postale
pub fn validate_address(address: &Address) -> Result<(), DeliveryError> {
    // Vérifie que les champs obligatoires sont remplis
    if address.name.trim().is_empty() {
        return Err(DeliveryError::InvalidAddress("Le nom est obligatoire".to_string()));
    }

    if address.street1.trim().is_empty() {
        return Err(DeliveryError::InvalidAddress("L'adresse est obligatoire".to_string()));
    }

    if address.city.trim().is_empty() {
        return Err(DeliveryError::InvalidAddress("La ville est obligatoire".to_string()));
    }

    if address.postal_code.trim().is_empty() {
        return Err(DeliveryError::InvalidAddress("Le code postal est obligatoire".to_string()));
    }

    if address.country.trim().is_empty() {
        return Err(DeliveryError::InvalidAddress("Le pays est obligatoire".to_string()));
    }

    // Règles de validation spécifiques par pays
    validate_country_specific(address)
}

/// Validations spécifiques par pays
fn validate_country_specific(address: &Address) -> Result<(), DeliveryError> {
    match address.country.as_str() {
        "FR" => validate_address_france(address),
        "US" => validate_address_usa(address),
        // Ajouter d'autres pays au besoin
        _ => Ok(()),  // Pas de validation spécifique pour les autres pays
    }
}

/// Validations spécifiques pour la France
fn validate_address_france(address: &Address) -> Result<(), DeliveryError> {
    // Valide le format du code postal français (5 chiffres)
    if !address.postal_code.chars().all(char::is_numeric) || address.postal_code.len() != 5 {
        return Err(DeliveryError::InvalidAddress(
            "Le code postal français doit contenir 5 chiffres".to_string()
        ));
    }

    // Vérifie que le numéro de téléphone est au format français si présent
    if let Some(phone) = &address.phone {
        if !phone.starts_with("+33") && !phone.starts_with("0") {
            return Err(DeliveryError::InvalidAddress(
                "Le numéro de téléphone français doit commencer par +33 ou 0".to_string()
            ));
        }
    }

    Ok(())
}

/// Validations spécifiques pour les États-Unis
fn validate_address_usa(address: &Address) -> Result<(), DeliveryError> {
    // Vérifie que l'état est présent (obligatoire aux USA)
    if address.state.is_none() || address.state.as_ref().unwrap().trim().is_empty() {
        return Err(DeliveryError::InvalidAddress(
            "L'état est obligatoire pour les adresses aux États-Unis".to_string()
        ));
    }

    // Valide le format du code postal américain (5 chiffres ou 5+4)
    let postal = &address.postal_code;
    let is_valid = (postal.len() == 5 && postal.chars().all(char::is_numeric)) ||
                   (postal.len() == 10 && postal.chars().enumerate().all(|(i, c)| {
                       (i != 5 && c.is_numeric()) || (i == 5 && c == '-')
                   }));

    if !is_valid {
        return Err(DeliveryError::InvalidAddress(
            "Le code postal américain doit être au format 12345 ou 12345-6789".to_string()
        ));
    }

    Ok(())
}

/// Valide un colis pour l'expédition
pub fn validate_parcel(parcel: &Parcel) -> Result<(), DeliveryError> {
    // Vérifie les dimensions et le poids
    if parcel.weight <= 0.0 {
        return Err(DeliveryError::InvalidParcel(
            "Le poids du colis doit être supérieur à 0".to_string()
        ));
    }

    if parcel.length <= 0.0 || parcel.width <= 0.0 || parcel.height <= 0.0 {
        return Err(DeliveryError::InvalidParcel(
            "Les dimensions du colis doivent être supérieures à 0".to_string()
        ));
    }

    // Valide les adresses
    validate_address(&parcel.sender)?;
    validate_address(&parcel.recipient)?;

    // Vérifie que l'expéditeur et le destinataire sont différents
    if parcel.sender.postal_code == parcel.recipient.postal_code &&
       parcel.sender.street1 == parcel.recipient.street1 &&
       parcel.sender.name == parcel.recipient.name {
        return Err(DeliveryError::InvalidParcel(
            "L'expéditeur et le destinataire ne peuvent pas être identiques".to_string()
        ));
    }

    Ok(())
}

/// Valide un numéro de suivi
pub fn validate_tracking_number(tracking: &str, carrier_code: Option<crate::models::CarrierCode>) -> bool {
    if tracking.trim().is_empty() {
        return false;
    }

    match carrier_code {
        Some(crate::models::CarrierCode::Colissimo) => {
            // Exemple: 13 caractères se terminant par FR
            tracking.len() == 13 && tracking.ends_with("FR")
        }
        Some(crate::models::CarrierCode::Chronopost) => {
            // Exemple: 10 caractères commençant par CH
            tracking.len() == 10 && tracking.starts_with("CH")
        }
        Some(crate::models::CarrierCode::FedEx) => {
            // Exemple: 12 chiffres
            tracking.len() == 12 && tracking.chars().all(|c| c.is_numeric())
        }
        Some(crate::models::CarrierCode::UPS) => {
            // Exemple: 1Z suivi de caractères alphanumériques
            tracking.len() == 18 && tracking.starts_with("1Z")
        }
        Some(crate::models::CarrierCode::DHL) => {
            // Exemple: 10 chiffres
            tracking.len() == 10 && tracking.chars().all(|c| c.is_numeric())
        }
        None => {
            // Validation basique sans spécifier le transporteur
            !tracking.is_empty() && tracking.len() >= 8 && tracking.len() <= 30
        }
    }
}
