use crate::types::{Shipment, ShippingRate, LabelResponse, TrackingResponse, TrackingEvent};
use crate::error::ShippingError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize)]
struct ColissimoRateRequest {
    #[serde(rename = "sender")]
    sender: ColissimoAddress,
    #[serde(rename = "recipient")]
    recipient: ColissimoAddress,
    #[serde(rename = "parcel")]
    parcel: ColissimoParcel,
}

#[derive(Serialize)]
struct ColissimoAddress {
    #[serde(rename = "zipCode")]
    zip_code: String,
    #[serde(rename = "countryCode")]
    country_code: String,
}

#[derive(Serialize)]
struct ColissimoParcel {
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
struct ColissimoRateResponse {
    #[serde(rename = "rates")]
    rates: Vec<ColissimoRate>,
}

#[derive(Deserialize)]
struct ColissimoRate {
    #[serde(rename = "serviceName")]
    service_name: String,
    #[serde(rename = "price")]
    price: f32,
    #[serde(rename = "estimatedDeliveryDays")]
    estimated_delivery_days: Option<u32>,
}

#[derive(Serialize)]
struct ColissimoLabelRequest {
    #[serde(rename = "contractNumber")]
    contract_number: String,
    #[serde(rename = "password")]
    password: String,
    #[serde(rename = "sender")]
    sender: ColissimoAddress,
    #[serde(rename = "recipient")]
    recipient: ColissimoAddress,
    #[serde(rename = "parcel")]
    parcel: ColissimoParcel,
    #[serde(rename = "service")]
    service: ColissimoService,
}

#[derive(Serialize)]
struct ColissimoService {
    #[serde(rename = "productCode")]
    product_code: String,
    #[serde(rename = "labelFormat")]
    label_format: String,
}

#[derive(Deserialize)]
struct ColissimoLabelResponse {
    #[serde(rename = "trackingNumber")]
    tracking_number: String,
    #[serde(rename = "labelUrl")]
    label_url: String,
    #[serde(rename = "expiryDate")]
    expiry_date: Option<String>,
}

#[derive(Serialize)]
struct ColissimoTrackRequest {
    #[serde(rename = "trackingNumber")]
    tracking_number: String,
}

#[derive(Deserialize)]
struct ColissimoTrackResponse {
    #[serde(rename = "status")]
    status: String,
    #[serde(rename = "lastUpdate")]
    last_update: String,
    #[serde(rename = "events")]
    events: Vec<ColissimoEvent>,
}

#[derive(Deserialize)]
struct ColissimoEvent {
    #[serde(rename = "eventDate")]
    event_date: String,
    #[serde(rename = "location")]
    location: String,
    #[serde(rename = "description")]
    description: String,
}

// Helper function for async retries
async fn async_retry<F, Fut, T>(
    max_attempts: usize,
    delay_ms: u64,
    mut operation: F,
) -> Result<T, ShippingError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, ShippingError>>,
{
    let mut attempts = 0;
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempts < max_attempts - 1 => {
                attempts += 1;
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                log::warn!("Tentative {} échouée : {}. Réessai...", attempts, e);
            }
            Err(e) => return Err(ShippingError::ApiRequestFailed(format!("Échec après {} tentatives : {}", attempts + 1, e))),
        }
    }
}

pub async fn get_rates(shipment: &Shipment) -> Result<Vec<ShippingRate>, ShippingError> {
    let api_key = std::env::var("COLISSIMO_API_KEY")
        .map_err(|_| ShippingError::EnvVarMissing("COLISSIMO_API_KEY".to_string()))?;
    let url = std::env::var("URL_API_CARRIER_COLISSIMO")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_COLISSIMO".to_string()))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;

    let payload = ColissimoRateRequest {
        sender: ColissimoAddress {
            zip_code: shipment.sender.postal_code.clone(),
            country_code: shipment.sender.country_code.clone(),
        },
        recipient: ColissimoAddress {
            zip_code: shipment.recipient.postal_code.clone(),
            country_code: shipment.recipient.country_code.clone(),
        },
        parcel: ColissimoParcel {
            weight: shipment.package.weight_kg,
            length: shipment.package.length_cm,
            width: shipment.package.width_cm,
            height: shipment.package.height_cm,
        },
    };

    let operation = || async {
        client
            .post(format!("{}/rates", url))
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))
    };

    let res = async_retry(3, 1000, operation).await?;

    if !res.status().is_success() {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur API Colissimo : {}", res.status())));
    }

    let data: ColissimoRateResponse = res
        .json()
        .await
        .map_err(|e| ShippingError::JsonParsingError(e.to_string()))?;

    let rates = data.rates.into_iter().map(|r| ShippingRate {
        service_name: r.service_name,
        price_eur: r.price,
        estimated_days: r.estimated_delivery_days,
    }).collect();

    Ok(rates)
}

pub async fn create_label(shipment: &Shipment) -> Result<LabelResponse, ShippingError> {
    let api_key = std::env::var("COLISSIMO_API_KEY")
        .map_err(|_| ShippingError::EnvVarMissing("COLISSIMO_API_KEY".to_string()))?;
    let url = std::env::var("URL_API_CARRIER_COLISSIMO")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_COLISSIMO".to_string()))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;

    let payload = ColissimoLabelRequest {
        contract_number: std::env::var("COLISSIMO_CONTRACT_NUMBER")
            .map_err(|_| ShippingError::EnvVarMissing("COLISSIMO_CONTRACT_NUMBER".to_string()))?,
        password: std::env::var("COLISSIMO_PASSWORD")
            .map_err(|_| ShippingError::EnvVarMissing("COLISSIMO_PASSWORD".to_string()))?,
        sender: ColissimoAddress {
            zip_code: shipment.sender.postal_code.clone(),
            country_code: shipment.sender.country_code.clone(),
        },
        recipient: ColissimoAddress {
            zip_code: shipment.recipient.postal_code.clone(),
            country_code: shipment.recipient.country_code.clone(),
        },
        parcel: ColissimoParcel {
            weight: shipment.package.weight_kg,
            length: shipment.package.length_cm,
            width: shipment.package.width_cm,
            height: shipment.package.height_cm,
        },
        service: ColissimoService {
            product_code: "COL".to_string(), // Exemple : Colissimo Domicile
            label_format: "PDF".to_string(),
        },
    };

    let operation = || async {
        client
            .post(format!("{}/labels", url))
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))
    };

    let res = async_retry(3, 1000, operation).await?;

    if !res.status().is_success() {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur API Colissimo : {}", res.status())));
    }

    let data: ColissimoLabelResponse = res
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
    let api_key = std::env::var("COLISSIMO_API_KEY")
        .map_err(|_| ShippingError::EnvVarMissing("COLISSIMO_API_KEY".to_string()))?;
    let url = std::env::var("URL_API_CARRIER_COLISSIMO")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_COLISSIMO".to_string()))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;

    let payload = ColissimoTrackRequest {
        tracking_number: tracking_number.to_string(),
    };

    let operation = || async {
        client
            .post(format!("{}/track", url))
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))
    };

    let res = async_retry(3, 1000, operation).await?;

    if !res.status().is_success() {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur API Colissimo : {}", res.status())));
    }

    let data: ColissimoTrackResponse = res
        .json()
        .await
        .map_err(|e| ShippingError::JsonParsingError(e.to_string()))?;

    Ok(TrackingResponse {
        status: data.status,
        last_updated: data.last_update,
        events: data.events.into_iter().map(|e| TrackingEvent {
            date: e.event_date,
            location: e.location,
            description: e.description,
        }).collect(),
    })
}