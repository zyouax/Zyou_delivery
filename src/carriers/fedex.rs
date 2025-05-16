use crate::types::{Shipment, ShippingRate, LabelResponse, TrackingResponse, TrackingEvent};
use crate::error::ShippingError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use retry::{delay::Fixed, retry};
use std::time::Duration;

#[derive(Serialize)]
struct FedExAuthRequest {
    grant_type: String,
    client_id: String,
    client_secret: String,
}

#[derive(Deserialize)]
struct FedExAuthResponse {
    access_token: String,
    expires_in: u32,
}

#[derive(Serialize)]
struct FedExRateRequest {
    #[serde(rename = "accountNumber")]
    account_number: FedExAccountNumber,
    #[serde(rename = "requestedShipment")]
    requested_shipment: FedExRequestedShipment,
}

#[derive(Serialize)]
struct FedExAccountNumber {
    value: String,
}

#[derive(Serialize)]
struct FedExRequestedShipment {
    #[serde(rename = "shipper")]
    shipper: FedExAddress,
    #[serde(rename = "recipient")]
    recipient: FedExAddress,
    #[serde(rename = "package")]
    package: FedExPackage,
    #[serde(rename = "serviceType")]
    service_type: String,
}

#[derive(Serialize)]
struct FedExAddress {
    #[serde(rename = "postalCode")]
    postal_code: String,
    #[serde(rename = "countryCode")]
    country_code: String,
}

#[derive(Serialize)]
struct FedExPackage {
    weight: FedExWeight,
    dimensions: FedExDimensions,
}

#[derive(Serialize)]
struct FedExWeight {
    units: String,
    value: f32,
}

#[derive(Serialize)]
struct FedExDimensions {
    units: String,
    length: f32,
    width: f32,
    height: f32,
}

#[derive(Deserialize)]
struct FedExRateResponse {
    #[serde(rename = "output")]
    output: FedExRateOutput,
}

#[derive(Deserialize)]
struct FedExRateOutput {
    #[serde(rename = "rateReplyDetails")]
    rate_reply_details: Vec<FedExRateDetail>,
}

#[derive(Deserialize)]
struct FedExRateDetail {
    #[serde(rename = "serviceType")]
    service_type: String,
    #[serde(rename = "ratedShipmentDetails")]
    rated_shipment_details: Vec<FedExShipmentDetail>,
    #[serde(rename = "deliveryTimestamp")]
    delivery_timestamp: Option<String>,
}

#[derive(Deserialize)]
struct FedExShipmentDetail {
    #[serde(rename = "totalNetCharge")]
    total_net_charge: f32,
}

#[derive(Serialize)]
struct FedExLabelRequest {
    #[serde(rename = "accountNumber")]
    account_number: FedExAccountNumber,
    #[serde(rename = "requestedShipment")]
    requested_shipment: FedExLabelShipment,
}

#[derive(Serialize)]
struct FedExLabelShipment {
    #[serde(rename = "shipper")]
    shipper: FedExAddress,
    #[serde(rename = "recipient")]
    recipient: FedExAddress,
    #[serde(rename = "package")]
    package: FedExPackage,
    #[serde(rename = "serviceType")]
    service_type: String,
    #[serde(rename = "labelSpecification")]
    label_specification: FedExLabelSpecification,
}

#[derive(Serialize)]
struct FedExLabelSpecification {
    #[serde(rename = "labelFormatType")]
    label_format_type: String,
}

#[derive(Deserialize)]
struct FedExLabelResponse {
    #[serde(rename = "output")]
    output: FedExLabelOutput,
}

#[derive(Deserialize)]
struct FedExLabelOutput {
    #[serde(rename = "trackingNumber")]
    tracking_number: String,
    #[serde(rename = "label")]
    label: FedExLabel,
}

#[derive(Deserialize)]
struct FedExLabel {
    #[serde(rename = "labelURL")]
    label_url: String,
    #[serde(rename = "expiryDate")]
    expiry_date: Option<String>,
}

#[derive(Serialize)]
struct FedExTrackRequest {
    #[serde(rename = "trackingInfo")]
    tracking_info: FedExTrackingInfo,
}

#[derive(Serialize)]
struct FedExTrackingInfo {
    #[serde(rename = "trackingNumber")]
    tracking_number: String,
}

#[derive(Deserialize)]
struct FedExTrackResponse {
    #[serde(rename = "output")]
    output: FedExTrackOutput,
}

#[derive(Deserialize)]
struct FedExTrackOutput {
    #[serde(rename = "completeTrackResults")]
    complete_track_results: Vec<FedExTrackResult>,
}

#[derive(Deserialize)]
struct FedExTrackResult {
    #[serde(rename = "trackResults")]
    track_results: Vec<FedExTrackDetail>,
}

#[derive(Deserialize)]
struct FedExTrackDetail {
    #[serde(rename = "latestStatusDetail")]
    latest_status_detail: FedExStatusDetail,
    #[serde(rename = "scanEvents")]
    scan_events: Vec<FedExScanEvent>,
}

#[derive(Deserialize)]
struct FedExStatusDetail {
    status: String,
    #[serde(rename = "statusDate")]
    status_date: String,
}

#[derive(Deserialize)]
struct FedExScanEvent {
    date: String,
    #[serde(rename = "scanLocation")]
    scan_location: String,
    #[serde(rename = "eventDescription")]
    event_description: String,
}

async fn get_oauth_token() -> Result<String, ShippingError> {
    let client_id = std::env::var("FEDEX_CLIENT_ID")
        .map_err(|_| ShippingError::EnvVarMissing("FEDEX_CLIENT_ID".to_string()))?;
    let client_secret = std::env::var("FEDEX_CLIENT_SECRET")
        .map_err(|_| ShippingError::EnvVarMissing("FEDEX_CLIENT_SECRET".to_string()))?;
    let url = std::env::var("URL_API_CARRIER_FEDEX")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_FEDEX".to_string()))?;
    let client = Client::new();

    let payload = FedExAuthRequest {
        grant_type: "client_credentials".to_string(),
        client_id,
        client_secret,
    };

    let operation = || async {
        client
            .post(format!("{}/oauth/token", url))
            .form(&payload)
            .send()
            .await
            .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))
    };

    let res = retry(Fixed::from_millis(1000).take(3), operation)
        .await
        .map_err(|e| ShippingError::ApiRequestFailed(format!("Échec après retries : {}", e)))?;

    if !res.status().is_success() {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur OAuth FedEx : {}", res.status())));
    }

    let data: FedExAuthResponse = res
        .json()
        .await
        .map_err(|e| ShippingError::JsonParsingError(e.to_string()))?;

    Ok(data.access_token)
}

pub async fn get_rates(shipment: &Shipment) -> Result<Vec<ShippingRate>, ShippingError> {
    let token = get_oauth_token().await?;
    let url = std::env::var("URL_API_CARRIER_FEDEX")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_FEDEX".to_string()))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;

    let payload = FedExRateRequest {
        account_number: FedExAccountNumber {
            value: std::env::var("FEDEX_ACCOUNT_NUMBER")
                .map_err(|_| ShippingError::EnvVarMissing("FEDEX_ACCOUNT_NUMBER".to_string()))?,
        },
        requested_shipment: FedExRequestedShipment {
            shipper: FedExAddress {
                postal_code: shipment.sender.postal_code.clone(),
                country_code: shipment.sender.country_code.clone(),
            },
            recipient: FedExAddress {
                postal_code: shipment.recipient.postal_code.clone(),
                country_code: shipment.recipient.country_code.clone(),
            },
            package: FedExPackage {
                weight: FedExWeight {
                    units: "KG".to_string(),
                    value: shipment.package.weight_kg,
                },
                dimensions: FedExDimensions {
                    units: "CM".to_string(),
                    length: shipment.package.length_cm,
                    width: shipment.package.width_cm,
                    height: shipment.package.height_cm,
                },
            },
            service_type: "STANDARD_OVERNIGHT".to_string(), // Exemple, peut être paramétré
        },
    };

    let operation = || async {
        client
            .post(format!("{}/rate/v1/rates/quotes", url))
            .bearer_auth(&token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))
    };

    let res = retry(Fixed::from_millis(1000).take(3), operation)
        .await
        .map_err(|e| ShippingError::ApiRequestFailed(format!("Échec après retries : {}", e)))?;

    if !res.status().is_success() {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur API FedEx : {}", res.status())));
    }

    let data: FedExRateResponse = res
        .json()
        .await
        .map_err(|e| ShippingError::JsonParsingError(e.to_string()))?;

    let rates = data.output.rate_reply_details.into_iter().map(|d| {
        ShippingRate {
            service_name: d.service_type,
            price_eur: d.rated_shipment_details.get(0).map_or(0.0, |s| s.total_net_charge),
            estimated_days: d.delivery_timestamp.and_then(|t| {
                chrono::DateTime::parse_from_rfc3339(&t)
                    .ok()
                    .map(|dt| (dt - chrono::Utc::now()).num_days() as u32)
            }),
        }
    }).collect();

    Ok(rates)
}

pub async fn create_label(shipment: &Shipment) -> Result<LabelResponse, ShippingError> {
    let token = get_oauth_token().await?;
    let url = std::env::var("URL_API_CARRIER_FEDEX")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_FEDEX".to_string()))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;

    let payload = FedExLabelRequest {
        account_number: FedExAccountNumber {
            value: std::env::var("FEDEX_ACCOUNT_NUMBER")
                .map_err(|_| ShippingError::EnvVarMissing("FEDEX_ACCOUNT_NUMBER".to_string()))?,
        },
        requested_shipment: FedExLabelShipment {
            shipper: FedExAddress {
                postal_code: shipment.sender.postal_code.clone(),
                country_code: shipment.sender.country_code.clone(),
            },
            recipient: FedExAddress {
                postal_code: shipment.recipient.postal_code.clone(),
                country_code: shipment.recipient.country_code.clone(),
            },
            package: FedExPackage {
                weight: FedExWeight {
                    units: "KG".to_string(),
                    value: shipment.package.weight_kg,
                },
                dimensions: FedExDimensions {
                    units: "CM".to_string(),
                    length: shipment.package.length_cm,
                    width: shipment.package.width_cm,
                    height: shipment.package.height_cm,
                },
            },
            service_type: "STANDARD_OVERNIGHT".to_string(),
            label_specification: FedExLabelSpecification {
                label_format_type: "PDF".to_string(),
            },
        },
    };

    let operation = || async {
        client
            .post(format!("{}/ship/v1/shipments", url))
            .bearer_auth(&token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))
    };

    let res = retry(Fixed::from_millis(1000).take(3), operation)
        .await
        .map_err(|e| ShippingError::ApiRequestFailed(format!("Échec après retries : {}", e)))?;

    if !res.status().is_success() {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur API FedEx : {}", res.status())));
    }

    let data: FedExLabelResponse = res
        .json()
        .await
        .map_err(|e| ShippingError::JsonParsingError(e.to_string()))?;

    Ok(LabelResponse {
        label_url: data.output.label.label_url,
        tracking_number: data.output.tracking_number,
        expiry_date: data.output.label.expiry_date,
    })
}

pub async fn track_package(tracking_number: &str) -> Result<TrackingResponse, ShippingError> {
    let token = get_oauth_token().await?;
    let url = std::env::var("URL_API_CARRIER_FEDEX")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_FEDEX".to_string()))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;

    let payload = FedExTrackRequest {
        tracking_info: FedExTrackingInfo {
            tracking_number: tracking_number.to_string(),
        },
    };

    let operation = || async {
        client
            .post(format!("{}/track/v1/tracking", url))
            .bearer_auth(&token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))
    };

    let res = retry(Fixed::from_millis(1000).take(3), operation)
        .await
        .map_err(|e| ShippingError::ApiRequestFailed(format!("Échec après retries : {}", e)))?;

    if !res.status().is_success() {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur API FedEx : {}", res.status())));
    }

    let data: FedExTrackResponse = res
        .json()
        .await
        .map_err(|e| ShippingError::JsonParsingError(e.to_string()))?;

    let track_result = data.output.complete_track_results
        .get(0)
        .and_then(|r| r.track_results.get(0))
        .ok_or_else(|| ShippingError::JsonParsingError("Aucun résultat de suivi trouvé".to_string()))?;

    Ok(TrackingResponse {
        status: track_result.latest_status_detail.status.clone(),
        last_updated: track_result.latest_status_detail.status_date.clone(),
        events: track_result.scan_events.iter().map(|e| TrackingEvent {
            date: e.date.clone(),
            location: e.scan_location.clone(),
            description: e.event_description.clone(),
        }).collect(),
    })
}