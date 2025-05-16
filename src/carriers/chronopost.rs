use crate::types::{Shipment, ShippingRate, LabelResponse, TrackingResponse, TrackingEvent};
use crate::error::ShippingError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use retry::{delay::Fixed, retry};
use std::time::Duration;

#[derive(Serialize)]
struct ChronopostRateRequest {
    #[serde(rename = "accountNumber")]
    account_number: String,
    #[serde(rename = "password")]
    password: String,
    #[serde(rename = "depCode")]
    dep_code: String,
    #[serde(rename = "arrCode")]
    arr_code: String,
    #[serde(rename = "weight")]
    weight: f32,
    #[serde(rename = "productCode")]
    product_code: String,
}

#[derive(Deserialize)]
struct ChronopostRateResponse {
    #[serde(rename = "priceDetails")]
    price_details: Vec<ChronopostPriceDetail>,
}

#[derive(Deserialize)]
struct ChronopostPriceDetail {
    #[serde(rename = "productCode")]
    product_code: String,
    #[serde(rename = "amount")]
    amount: f32,
    #[serde(rename = "deliveryDelay")]
    delivery_delay: Option<u32>,
}

#[derive(Serialize)]
struct ChronopostLabelRequest {
    #[serde(rename = "accountNumber")]
    account_number: String,
    #[serde(rename = "password")]
    password: String,
    #[serde(rename = "shipperAddress")]
    shipper_address: ChronopostAddress,
    #[serde(rename = "customerAddress")]
    customer_address: ChronopostAddress,
    #[serde(rename = "parcel")]
    parcel: ChronopostParcel,
    #[serde(rename = "productCode")]
    product_code: String,
    #[serde(rename = "labelFormat")]
    label_format: String,
}

#[derive(Serialize)]
struct ChronopostAddress {
    #[serde(rename = "zipCode")]
    zip_code: String,
    #[serde(rename = "countryCode")]
    country_code: String,
}

#[derive(Serialize)]
struct ChronopostParcel {
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
struct ChronopostLabelResponse {
    #[serde(rename = "trackingNumber")]
    tracking_number: String,
    #[serde(rename = "labelUrl")]
    label_url: String,
    #[serde(rename = "expiryDate")]
    expiry_date: Option<String>,
}

#[derive(Deserialize)]
struct ChronopostTrackResponse {
    #[serde(rename = "status")]
    status: String,
    #[serde(rename = "lastUpdate")]
    last_update: String,
    #[serde(rename = "events")]
    events: Vec<ChronopostEvent>,
}

#[derive(Deserialize)]
struct ChronopostEvent {
    #[serde(rename = "eventDate")]
    event_date: String,
    #[serde(rename = "location")]
    location: String,
    #[serde(rename = "description")]
    description: String,
}

pub async fn get_rates(shipment: &Shipment) -> Result<Vec<ShippingRate>, ShippingError> {
    let account_number = std::env::var("CHRONOPOST_ACCOUNT_NUMBER")
        .map_err(|_| ShippingError::EnvVarMissing("CHRONOPOST_ACCOUNT_NUMBER".to_string()))?;
    let password = std::env::var("CHRONOPOST_PASSWORD")
        .map_err(|_| ShippingError::EnvVarMissing("CHRONOPOST_PASSWORD".to_string()))?;
    let url = std::env::var("URL_API_CARRIER_CHRONOPOST")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_CHRONOPOST".to_string()))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;

    let payload = ChronopostRateRequest {
        account_number,
        password,
        dep_code: shipment.sender.postal_code.clone(),
        arr_code: shipment.recipient.postal_code.clone(),
        weight: shipment.package.weight_kg,
        product_code: "01".to_string(), // Chrono 13
    };

    let operation = || async {
        client
            .post(format!("{}/quickcost/v1", url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))
    };

    let res = retry(Fixed::from_millis(1000).take(3), operation)
        .await
        .map_err(|e| ShippingError::ApiRequestFailed(format!("Échec après retries : {}", e)))?;

    if !res.status().is_success() {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur API Chronopost : {}", res.status())));
    }

    let data: ChronopostRateResponse = res
        .json()
        .await
        .map_err(|e| ShippingError::JsonParsingError(e.to_string()))?;

    let rates = data.price_details.into_iter().map(|d| ShippingRate {
        service_name: d.product_code,
        price_eur: d.amount,
        estimated_days: d.delivery_delay,
    }).collect();

    Ok(rates)
}

pub async fn create_label(shipment: &Shipment) -> Result<LabelResponse, ShippingError> {
    let account_number = std::env::var("CHRONOPOST_ACCOUNT_NUMBER")
        .map_err(|_| ShippingError::EnvVarMissing("CHRONOPOST_ACCOUNT_NUMBER".to_string()))?;
    let password = std::env::var("CHRONOPOST_PASSWORD")
        .map_err(|_| ShippingError::EnvVarMissing("CHRONOPOST_PASSWORD".to_string()))?;
    let url = std::env::var("URL_API_CARRIER_CHRONOPOST")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_CHRONOPOST".to_string()))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;

    let payload = ChronopostLabelRequest {
        account_number,
        password,
        shipper_address: ChronopostAddress {
            zip_code: shipment.sender.postal_code.clone(),
            country_code: shipment.sender.country_code.clone(),
        },
        customer_address: ChronopostAddress {
            zip_code: shipment.recipient.postal_code.clone(),
            country_code: shipment.recipient.country_code.clone(),
        },
        parcel: ChronopostParcel {
            weight: shipment.package.weight_kg,
            length: shipment.package.length_cm,
            width: shipment.package.width_cm,
            height: shipment.package.height_cm,
        },
        product_code: "01".to_string(),
        label_format: "PDF".to_string(),
    };

    let operation = || async {
        client
            .post(format!("{}/shipping/v1", url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))
    };

    let res = retry(Fixed::from_millis(1000).take(3), operation)
        .await
        .map_err(|e| ShippingError::ApiRequestFailed(format!("Échec après retries : {}", e)))?;

    if !res.status().is_success() {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur API Chronopost : {}", res.status())));
    }

    let data: ChronopostLabelResponse = res
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
    let account_number = std::env::var("CHRONOPOST_ACCOUNT_NUMBER")
        .map_err(|_| ShippingError::EnvVarMissing("CHRONOPOST_ACCOUNT_NUMBER".to_string()))?;
    let password = std::env::var("CHRONOPOST_PASSWORD")
        .map_err(|_| ShippingError::EnvVarMissing("CHRONOPOST_PASSWORD".to_string()))?;
    let url = std::env::var("URL_API_CARRIER_CHRONOPOST")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_CHRONOPOST".to_string()))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;

    let operation = || async {
        client
            .get(format!("{}/tracking/v1?accountNumber={}&password={}&trackingNumber={}",
                url, account_number, password, tracking_number))
            .send()
            .await
            .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))
    };

    let res = retry(Fixed::from_millis(1000).take(3), operation)
        .await
        .map_err(|e| ShippingError::ApiRequestFailed(format!("Échec après retries : {}", e)))?;

    if !res.status().is_success() {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur API Chronopost : {}", res.status())));
    }

    let data: ChronopostTrackResponse = res
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