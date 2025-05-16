use crate::types::{Shipment, ShippingRate, LabelResponse, TrackingResponse, TrackingEvent};
use crate::error::ShippingError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use retry::{delay::Fixed, retry};
use std::time::Duration;

#[derive(Serialize)]
struct TNTRateRequest {
    #[serde(rename = "sender")]
    sender: TNTAddress,
    #[serde(rename = "receiver")]
    receiver: TNTAddress,
    #[serde(rename = "shipmentDetails")]
    shipment_details: TNTShipmentDetails,
}

#[derive(Serialize)]
struct TNTAddress {
    #[serde(rename = "postalCode")]
    postal_code: String,
    #[serde(rename = "country")]
    country: String,
}

#[derive(Serialize)]
struct TNTShipmentDetails {
    #[serde(rename = "weight")]
    weight: f32,
    #[serde(rename = "length")]
    length: f32,
    #[serde(rename = "width")]
    width: f32,
    #[serde(rename = "height")]
    height: f32,
}

#[derive(Deserialize)]
struct TNTRateResponse {
    #[serde(rename = "priceDetails")]
    price_details: Vec<TNTPriceDetail>,
}

#[derive(Deserialize)]
struct TNTPriceDetail {
    #[serde(rename = "serviceCode")]
    service_code: String,
    #[serde(rename = "totalPrice")]
    total_price: f32,
    #[serde(rename = "estimatedDelivery")]
    estimated_delivery: Option<String>,
}

#[derive(Serialize)]
struct TNTLabelRequest {
    #[serde(rename = "account")]
    account: TNTAccount,
    #[serde(rename = "sender")]
    sender: TNTAddress,
    #[serde(rename = "receiver")]
    receiver: TNTAddress,
    #[serde(rename = "shipmentDetails")]
    shipment_details: TNTShipmentDetails,
    #[serde(rename = "labelFormat")]
    label_format: String,
}

#[derive(Serialize)]
struct TNTAccount {
    #[serde(rename = "accountNumber")]
    account_number: String,
}

#[derive(Deserialize)]
struct TNTLabelResponse {
    #[serde(rename = "trackingNumber")]
    tracking_number: String,
    #[serde(rename = "labelURL")]
    label_url: String,
    #[serde(rename = "expiryDate")]
    expiry_date: Option<String>,
}

#[derive(Deserialize)]
struct TNTTrackResponse {
    #[serde(rename = "consignment")]
    consignment: TNTConsignment,
}

#[derive(Deserialize)]
struct TNTConsignment {
    #[serde(rename = "status")]
    status: String,
    #[serde(rename = "lastUpdate")]
    last_update: String,
    #[serde(rename = "events")]
    events: Vec<TNTEvent>,
}

#[derive(Deserialize)]
struct TNTEvent {
    #[serde(rename = "eventDate")]
    event_date: String,
    #[serde(rename = "location")]
    location: String,
    #[serde(rename = "description")]
    description: String,
}

pub async fn get_rates(shipment: &Shipment) -> Result<Vec<ShippingRate>, ShippingError> {
    let api_key = std::env::var("TNT_API_KEY")
        .map_err(|_| ShippingError::EnvVarMissing("TNT_API_KEY".to_string()))?;
    let url = std::env::var("URL_API_CARRIER_TNT")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_TNT".to_string()))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;

    let payload = TNTRateRequest {
        sender: TNTAddress {
            postal_code: shipment.sender.postal_code.clone(),
            country: shipment.sender.country_code.clone(),
        },
        receiver: TNTAddress {
            postal_code: shipment.recipient.postal_code.clone(),
            country: shipment.recipient.country_code.clone(),
        },
        shipment_details: TNTShipmentDetails {
            weight: shipment.package.weight_kg,
            length: shipment.package.length_cm,
            width: shipment.package.width_cm,
            height: shipment.package.height_cm,
        },
    };

    let operation = || async {
        client
            .post(format!("{}/pricing", url))
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))
    };

    let res = retry(Fixed::from_millis(1000).take(3), operation)
        .await
        .map_err(|e| ShippingError::ApiRequestFailed(format!("Échec après retries : {}", e)))?;

    if !res.status().is_success() {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur API TNT : {}", res.status())));
    }

    let data: TNTRateResponse = res
        .json()
        .await
        .map_err(|e| ShippingError::JsonParsingError(e.to_string()))?;

    let rates = data.price_details.into_iter().map(|d| ShippingRate {
        service_name: d.service_code,
        price_eur: d.total_price,
        estimated_days: d.estimated_delivery
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t)
                .ok()
                .map(|dt| (dt - chrono::Utc::now()).num_days() as u32)),
    }).collect();

    Ok(rates)
}

pub async fn create_label(shipment: &Shipment) -> Result<LabelResponse, ShippingError> {
    let api_key = std::env::var("TNT_API_KEY")
        .map_err(|_| ShippingError::EnvVarMissing("TNT_API_KEY".to_string()))?;
    let url = std::env::var("URL_API_CARRIER_TNT")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_TNT".to_string()))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;

    let payload = TNTLabelRequest {
        account: TNTAccount {
            account_number: std::env::var("TNT_ACCOUNT_NUMBER")
                .map_err(|_| ShippingError::EnvVarMissing("TNT_ACCOUNT_NUMBER".to_string()))?,
        },
        sender: TNTAddress {
            postal_code: shipment.sender.postal_code.clone(),
            country: shipment.sender.country_code.clone(),
        },
        receiver: TNTAddress {
            postal_code: shipment.recipient.postal_code.clone(),
            country: shipment.recipient.country_code.clone(),
        },
        shipment_details: TNTShipmentDetails {
            weight: shipment.package.weight_kg,
            length: shipment.package.length_cm,
            width: shipment.package.width_cm,
            height: shipment.package.height_cm,
        },
        label_format: "PDF".to_string(),
    };

    let operation = || async {
        client
            .post(format!("{}/shipping", url))
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))
    };

    let res = retry(Fixed::from_millis(1000).take(3), operation)
        .await
        .map_err(|e| ShippingError::ApiRequestFailed(format!("Échec après retries : {}", e)))?;

    if !res.status().is_success() {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur API TNT : {}", res.status())));
    }

    let data: TNTLabelResponse = res
        .json()
        .await
        .map_err(|e| ShippingError::JsonParsingError(e.to_string()))?;

    Ok(LabelResponse {
        label_url: data.label_url,
        tracking_number: data.tracking_number,
        expiry_date: data.expiry_date,
    })
}

pub async fn track_package(tracking_number: &str) -> Result<TrackingResponse, ShippingError> {
    let api_key = std::env::var("TNT_API_KEY")
        .map_err(|_| ShippingError::EnvVarMissing("TNT_API_KEY".to_string()))?;
    let url = std::env::var("URL_API_CARRIER_TNT")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_TNT".to_string()))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;

    let operation = || async {
        client
            .get(format!("{}/track?trackingNumber={}", url, tracking_number))
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))
    };

    let res = retry(Fixed::from_millis(1000).take(3), operation)
        .await
        .map_err(|e| ShippingError::ApiRequestFailed(format!("Échec après retries : {}", e)))?;

    if !res.status().is_success() {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur API TNT : {}", res.status())));
    }

    let data: TNTTrackResponse = res
        .json()
        .await
        .map_err(|e| ShippingError::JsonParsingError(e.to_string()))?;

    Ok(TrackingResponse {
        status: data.consignment.status,
        last_updated: data.consignment.last_update,
        events: data.consignment.events.into_iter().map(|e| TrackingEvent {
            date: e.event_date,
            location: e.location,
            description: e.description,
        }).collect(),
    })
}