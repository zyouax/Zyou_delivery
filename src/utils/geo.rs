use std::collections::HashMap;

/// Structure pour contenir les informations sur un pays
#[derive(Debug, Clone)]
pub struct CountryInfo {
    pub code: String,
    pub name: String,
    pub postal_code_format: Option<String>,
    pub phone_code: String,
}

/// Obtient le nom d'un pays à partir de son code ISO à 2 lettres
pub fn country_name_from_code(code: &str) -> Option<String> {
    let countries = get_countries();
    countries.get(code).map(|country| country.name.clone())
}

/// Vérifie si un code pays est valide
pub fn is_valid_country_code(code: &str) -> bool {
    let countries = get_countries();
    countries.contains_key(code)
}

/// Calcule la distance approximative entre deux points géographiques (en km)
pub fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS: f64 = 6371.0; // rayon de la Terre en km

    let lat1_rad = lat1.to_radians();
    let lon1_rad = lon1.to_radians();
    let lat2_rad = lat2.to_radians();
    let lon2_rad = lon2.to_radians();

    let dlon = lon2_rad - lon1_rad;
    let dlat = lat2_rad - lat1_rad;

    let a = (dlat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS * c
}

/// Vérifie si une livraison est nationale (même pays pour l'expéditeur et le destinataire)
pub fn is_domestic_shipping(sender_country: &str, recipient_country: &str) -> bool {
    sender_country == recipient_country
}

/// Vérifie si une livraison est européenne (UE)
pub fn is_eu_shipping(sender_country: &str, recipient_country: &str) -> bool {
    let eu_countries = get_eu_countries();
    eu_countries.contains(&sender_country.to_uppercase().as_str()) &&
    eu_countries.contains(&recipient_country.to_uppercase().as_str())
}

/// Obtient la liste des pays de l'UE
fn get_eu_countries() -> Vec<&'static str> {
    vec![
        "AT", "BE", "BG", "HR", "CY", "CZ", "DK", "EE", "FI", "FR",
        "DE", "GR", "HU", "IE", "IT", "LV", "LT", "LU", "MT", "NL",
        "PL", "PT", "RO", "SK", "SI", "ES", "SE"
    ]
}

/// Obtient la carte des informations sur les pays
fn get_countries() -> HashMap<&'static str, CountryInfo> {
    let mut countries = HashMap::new();

    countries.insert(
        "FR",
        CountryInfo {
            code: "FR".to_string(),
            name: "France".to_string(),
            postal_code_format: Some(r"^\d{5}$".to_string()),
            phone_code: "33".to_string(),
        },
    );

    countries.insert(
        "US",
        CountryInfo {
            code: "US".to_string(),
            name: "United States".to_string(),
            postal_code_format: Some(r"^\d{5}(-\d{4})?$".to_string()),
            phone_code: "1".to_string(),
        },
    );

    countries.insert(
        "GB",
        CountryInfo {
            code: "GB".to_string(),
            name: "United Kingdom".to_string(),
            postal_code_format: Some(r"^[A-Z]{1,2}\d[A-Z\d]? \d[A-Z]{2}$".to_string()),
            phone_code: "44".to_string(),
        },
    );

    countries.insert(
        "DE",
        CountryInfo {
            code: "DE".to_string(),
            name: "Germany".to_string(),
            postal_code_format: Some(r"^\d{5}$".to_string()),
            phone_code: "49".to_string(),
        },
    );

    countries.insert(
        "IT",
        CountryInfo {
            code: "IT".to_string(),
            name: "Italy".to_string(),
            postal_code_format: Some(r"^\d{5}$".to_string()),
            phone_code: "39".to_string(),
        },
    );

    countries.insert(
        "ES",
        CountryInfo {
            code: "ES".to_string(),
            name: "Spain".to_string(),
            postal_code_format: Some(r"^\d{5}$".to_string()),
            phone_code: "34".to_string(),
        },
    );

    countries.insert(
        "BE",
        CountryInfo {
            code: "BE".to_string(),
            name: "Belgium".to_string(),
            postal_code_format: Some(r"^\d{4}$".to_string()),
            phone_code: "32".to_string(),
        },
    );

    countries.insert(
        "CA",
        CountryInfo {
            code: "CA".to_string(),
            name: "Canada".to_string(),
            postal_code_format: Some(r"^[A-Z]\d[A-Z] \d[A-Z]\d$".to_string()),
            phone_code: "1".to_string(),
        },
    );

    // Ajoutez d'autres pays selon les besoins...

    countries
}
