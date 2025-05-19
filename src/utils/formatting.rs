use crate::models::{Address, ShipmentStatus};

/// Formate une adresse pour l'affichage
pub fn format_address(address: &Address) -> String {
    let mut lines = Vec::new();

    lines.push(address.name.clone());

    if let Some(company) = &address.company {
        if !company.trim().is_empty() {
            lines.push(company.clone());
        }
    }

    lines.push(address.street1.clone());

    if let Some(street2) = &address.street2 {
        if !street2.trim().is_empty() {
            lines.push(street2.clone());
        }
    }

    let mut city_line = String::new();

    if let Some(state) = &address.state {
        city_line = format!("{} {} {}", address.postal_code, address.city, state);
    } else {
        city_line = format!("{} {}", address.postal_code, address.city);
    }

    lines.push(city_line);
    lines.push(address.country.clone());

    lines.join("\n")
}

/// Formate un numéro de téléphone en format international
pub fn format_phone(phone: &str, country: &str) -> String {
    match country {
        "FR" => format_phone_france(phone),
        "US" => format_phone_usa(phone),
        // Ajouter d'autres pays au besoin
        _ => phone.to_string(),
    }
}

/// Formate un numéro de téléphone français
fn format_phone_france(phone: &str) -> String {
    let digits: String = phone.chars().filter(|c| c.is_numeric()).collect();

    if digits.starts_with("33") && digits.len() >= 11 {
        // Format international: +33 1 23 45 67 89
        format!(
            "+{} {} {} {} {} {}",
            &digits[0..2],
            &digits[2..3],
            &digits[3..5],
            &digits[5..7],
            &digits[7..9],
            &digits[9..11]
        )
    } else if digits.starts_with("0") && digits.len() >= 10 {
        // Format national: 01 23 45 67 89
        format!(
            "{} {} {} {} {}",
            &digits[0..2],
            &digits[2..4],
            &digits[4..6],
            &digits[6..8],
            &digits[8..10]
        )
    } else {
        phone.to_string()
    }
}

/// Formate un numéro de téléphone américain
fn format_phone_usa(phone: &str) -> String {
    let digits: String = phone.chars().filter(|c| c.is_numeric()).collect();

    if digits.starts_with("1") && digits.len() >= 11 {
        // Format international: +1 (123) 456-7890
        format!(
            "+{} ({}) {}-{}",
            &digits[0..1],
            &digits[1..4],
            &digits[4..7],
            &digits[7..11]
        )
    } else if digits.len() >= 10 {
        // Format national: (123) 456-7890
        format!(
            "({}) {}-{}",
            &digits[0..3],
            &digits[3..6],
            &digits[6..10]
        )
    } else {
        phone.to_string()
    }
}

/// Standardise les codes de statut spécifiques aux transporteurs
pub fn normalize_status(carrier_status: &str, carrier_code: crate::models::CarrierCode) -> ShipmentStatus {
    match carrier_code {
        crate::models::CarrierCode::Colissimo => normalize_status_colissimo(carrier_status),
        crate::models::CarrierCode::Chronopost => normalize_status_chronopost(carrier_status),
        crate::models::CarrierCode::FedEx => normalize_status_fedex(carrier_status),
        crate::models::CarrierCode::UPS => normalize_status_ups(carrier_status),
        crate::models::CarrierCode::DHL => normalize_status_dhl(carrier_status),
    }
}

fn normalize_status_colissimo(status: &str) -> ShipmentStatus {
    match status {
        // Codes de base
        "PRC" => ShipmentStatus::Pickup,
        "PCH" => ShipmentStatus::Pickup,
        "PDR" => ShipmentStatus::InTransit,
        "PRES" => ShipmentStatus::OutForDelivery,
        "LIV" => ShipmentStatus::Delivered,
        "LIVRI" => ShipmentStatus::Delivered,
        "ANOML" => ShipmentStatus::Exception,
        "RET" => ShipmentStatus::Returned,

        // Codes supplémentaires identifiés dans l'API réelle
        "PCHMQT" => ShipmentStatus::Created,         // Colis en préparation
        "LIVCFM" => ShipmentStatus::Delivered,       // Livraison confirmée
        "AARBPR" => ShipmentStatus::OutForDelivery,  // Disponible en point de retrait
        "PRELIV" => ShipmentStatus::OutForDelivery,  // En préparation pour la livraison
        "PCHTRI" => ShipmentStatus::InTransit,       // En transit sur la plateforme

        // Code par défaut
        _ => ShipmentStatus::Unknown,
    }
}

fn normalize_status_chronopost(status: &str) -> ShipmentStatus {
    match status {
        "A" => ShipmentStatus::Created,
        "B" => ShipmentStatus::Pickup,
        "C" | "D" | "E" => ShipmentStatus::InTransit,
        "F" => ShipmentStatus::OutForDelivery,
        "G" => ShipmentStatus::Delivered,
        "H" => ShipmentStatus::Exception,
        "I" => ShipmentStatus::Returned,
        _ => ShipmentStatus::Unknown,
    }
}

fn normalize_status_fedex(status: &str) -> ShipmentStatus {
    match status {
        "AA" => ShipmentStatus::Created,
        "PU" => ShipmentStatus::Pickup,
        "IT" => ShipmentStatus::InTransit,
        "OD" => ShipmentStatus::OutForDelivery,
        "DL" => ShipmentStatus::Delivered,
        "DE" => ShipmentStatus::Exception,
        "RT" => ShipmentStatus::Returned,
        _ => ShipmentStatus::Unknown,
    }
}

fn normalize_status_ups(status: &str) -> ShipmentStatus {
    match status {
        "M" => ShipmentStatus::Created,
        "P" => ShipmentStatus::Pickup,
        "I" => ShipmentStatus::InTransit,
        "O" => ShipmentStatus::OutForDelivery,
        "D" => ShipmentStatus::Delivered,
        "X" => ShipmentStatus::Exception,
        "R" => ShipmentStatus::Returned,
        _ => ShipmentStatus::Unknown,
    }
}

fn normalize_status_dhl(status: &str) -> ShipmentStatus {
    match status {
        "SD" => ShipmentStatus::Created,
        "PU" => ShipmentStatus::Pickup,
        "IT" => ShipmentStatus::InTransit,
        "OD" => ShipmentStatus::OutForDelivery,
        "DL" => ShipmentStatus::Delivered,
        "EX" => ShipmentStatus::Exception,
        "RT" => ShipmentStatus::Returned,
        _ => ShipmentStatus::Unknown,
    }
}
