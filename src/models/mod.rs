use std::fmt;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Code d'identification pour les transporteurs supportés
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CarrierCode {
    Colissimo,
    Chronopost,
    FedEx,
    UPS,
    DHL,
    // Autres transporteurs à ajouter ici
}

impl fmt::Display for CarrierCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CarrierCode::Colissimo => write!(f, "Colissimo"),
            CarrierCode::Chronopost => write!(f, "Chronopost"),
            CarrierCode::FedEx => write!(f, "FedEx"),
            CarrierCode::UPS => write!(f, "UPS"),
            CarrierCode::DHL => write!(f, "DHL"),
        }
    }
}

/// Représente une adresse postale
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Address {
    pub name: String,
    pub company: Option<String>,
    pub street1: String,
    pub street2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: String,
    pub country: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

impl Address {
    /// Crée une nouvelle adresse avec les informations minimales requises
    pub fn new(name: &str, street: &str, postal_code: &str, city: &str, country: &str) -> Self {
        Self {
            name: name.to_string(),
            company: None,
            street1: street.to_string(),
            street2: None,
            city: city.to_string(),
            state: None,
            postal_code: postal_code.to_string(),
            country: country.to_string(),
            phone: None,
            email: None,
        }
    }

    /// Ajoute un nom d'entreprise à l'adresse
    pub fn with_company(mut self, company: &str) -> Self {
        self.company = Some(company.to_string());
        self
    }

    /// Ajoute une seconde ligne d'adresse
    pub fn with_street2(mut self, street2: &str) -> Self {
        self.street2 = Some(street2.to_string());
        self
    }

    /// Ajoute un état/région
    pub fn with_state(mut self, state: &str) -> Self {
        self.state = Some(state.to_string());
        self
    }

    /// Ajoute un numéro de téléphone
    pub fn with_phone(mut self, phone: &str) -> Self {
        self.phone = Some(phone.to_string());
        self
    }

    /// Ajoute une adresse email
    pub fn with_email(mut self, email: &str) -> Self {
        self.email = Some(email.to_string());
        self
    }
}

/// Représente un colis à expédier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parcel {
    pub id: Uuid,
    pub weight: f64,  // en kg
    pub length: f64,  // en cm
    pub width: f64,   // en cm
    pub height: f64,  // en cm
    pub sender: Address,
    pub recipient: Address,
    pub insurance_value: Option<f64>,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub is_return: bool,
}

impl Parcel {
    /// Crée un nouveau colis avec un identifiant unique
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            weight: 0.0,
            length: 0.0,
            width: 0.0,
            height: 0.0,
            sender: Address::new("", "", "", "", ""),
            recipient: Address::new("", "", "", "", ""),
            insurance_value: None,
            description: None,
            reference: None,
            is_return: false,
        }
    }

    /// Définit le poids du colis en kg
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    /// Définit les dimensions du colis en cm
    pub fn with_dimensions(mut self, length: f64, width: f64, height: f64) -> Self {
        self.length = length;
        self.width = width;
        self.height = height;
        self
    }

    /// Définit l'adresse de l'expéditeur
    pub fn with_sender(
        mut self,
        name: &str,
        street: &str,
        postal_code: &str,
        city: &str,
        country: &str
    ) -> Self {
        self.sender = Address::new(name, street, postal_code, city, country);
        self
    }

    /// Définit l'adresse du destinataire
    pub fn with_recipient(
        mut self,
        name: &str,
        street: &str,
        postal_code: &str,
        city: &str,
        country: &str
    ) -> Self {
        self.recipient = Address::new(name, street, postal_code, city, country);
        self
    }

    /// Définit la valeur d'assurance du colis
    pub fn with_insurance(mut self, value: f64) -> Self {
        self.insurance_value = Some(value);
        self
    }

    /// Ajoute une description au colis
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Ajoute une référence client au colis
    pub fn with_reference(mut self, reference: &str) -> Self {
        self.reference = Some(reference.to_string());
        self
    }

    /// Marque le colis comme un retour
    pub fn as_return(mut self) -> Self {
        self.is_return = true;
        self
    }
}

impl Default for Parcel {
    fn default() -> Self {
        Self::new()
    }
}

/// Représente une option de tarif proposée par un transporteur
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rate {
    pub id: String,
    pub carrier: CarrierCode,
    pub service: String,
    pub service_code: String,
    pub price: f64,
    pub currency: String,
    pub estimated_delivery: Option<DateTime<Utc>>,
    pub delivery_days: Option<u32>,
    pub guaranteed_delivery: bool,
    pub features: Vec<String>,  // Options comme signature, assurance, etc.
}

/// Status normalisé d'un colis en transit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShipmentStatus {
    Created,
    Pickup,
    InTransit,
    OutForDelivery,
    Delivered,
    Exception,
    Returned,
    Unknown,
}

impl fmt::Display for ShipmentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShipmentStatus::Created => write!(f, "Créé"),
            ShipmentStatus::Pickup => write!(f, "Pris en charge"),
            ShipmentStatus::InTransit => write!(f, "En transit"),
            ShipmentStatus::OutForDelivery => write!(f, "En cours de livraison"),
            ShipmentStatus::Delivered => write!(f, "Livré"),
            ShipmentStatus::Exception => write!(f, "Incident"),
            ShipmentStatus::Returned => write!(f, "Retourné"),
            ShipmentStatus::Unknown => write!(f, "Inconnu"),
        }
    }
}

/// Représente un événement de suivi d'un colis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackingEvent {
    pub timestamp: DateTime<Utc>,
    pub status: ShipmentStatus,
    pub location: Option<String>,
    pub description: String,
    pub raw_status: String,
}

/// Représente les informations complètes de suivi d'un colis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackingInfo {
    pub tracking_number: String,
    pub carrier: CarrierCode,
    pub status: ShipmentStatus,
    pub estimated_delivery: Option<DateTime<Utc>>,
    pub shipped_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub events: Vec<TrackingEvent>,
    pub signature_name: Option<String>,
}

/// Représente une étiquette d'expédition générée
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShippingLabel {
    pub carrier: CarrierCode,
    pub tracking_number: String,
    pub label_format: LabelFormat,
    pub label_data: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl ShippingLabel {
    /// Enregistre l'étiquette dans un fichier
    pub fn save_to_file(&self, path: &str) -> Result<(), std::io::Error> {
        std::fs::write(path, &self.label_data)
    }
}

/// Format de l'étiquette d'expédition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LabelFormat {
    PDF,
    ZPL,
    PNG,
}

/// Abstraction pour un transporteur
pub trait Carrier {
    /// Obtient le code du transporteur
    fn code(&self) -> CarrierCode;

    /// Obtient le nom du transporteur
    fn name(&self) -> String;
}
