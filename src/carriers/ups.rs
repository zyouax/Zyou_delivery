use crate::types::{Shipment, ShippingRate, LabelResponse, TrackingResponse, TrackingEvent};
use crate::error::ShippingError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use retry::{delay::Fixed, retry};
use std::time::Duration;

#[derive(Serialize)]
struct UPSAuthRequest {
    grant_type: String,
    client_id: String,
    client_secret: String,
}

#[derive(Deserialize)]
struct UPSAuthResponse {
    access_token: String,
}

#[derive(Serialize)]
struct UPSRateRequest {
    #[serde(rename = "RateRequest")]
    rate_request: UPSRateRequestDetails,
}

#[derive(Serialize)]
struct UPSRateRequestDetails {
    #[serde(rename = "Shipper")]
    shipper: UPSAddress,
    #[serde(rename = "ShipTo")]
    ship_to: UPSAddress,
    #[serde(rename = "Package")]
    package: UPSPackage,
}

#[derive(Serialize)]
struct UPSAddress {
    #[serde(rename = "PostalCode")]
    postal_code: String,
    #[serde(rename = "CountryCode")]
    country_code: String,
}

#[derive(Serialize)]
struct UPSPackage {
    #[serde(rename = "Weight")]
    weight: UPSWeight,
    #[serde(rename = "Dimensions")]
    dimensions: UPSDimensions,
}

#[derive(Serialize)]
struct UPSWeight {
    #[serde(rename = "UnitOfMeasurement")]
    unit: UPSUnit,
    #[serde(rename = "Weight")]
    weight: f32,
}

#[derive(Serialize)]
struct UPSUnit {
    #[serde(rename = "Code")]
    code: String,
}

#[derive(Serialize)]
struct UPSDimensions {
    #[serde(rename = "UnitOfMeasurement")]
    unit: UPSUnit,
    #[serde(rename = "Length")]
    length: f32,
    #[serde(rename = "Width")]
    width: f32,
    #[serde(rename = "Height")]
    height: f32,
}

#[derive(Deserialize)]
struct UPSRateResponse {
    #[serde(rename = "RateResponse")]
    rate_response: UPSRateResponseDetails,
}

#[derive(Deserialize)]
struct UPSRateResponseDetails {
    #[serde(rename = "RatedShipment")]
    rated_shipment: Vec<UPSRatedShipment>,
}

#[derive(Deserialize)]
struct UPSRatedShipment {
    #[serde(rename = "Service")]
    service: UPSService,
    #[serde(rename = "TotalCharges")]
    total_charges: UPSTotalCharges,
    #[serde(rename = "GuaranteedDelivery")]
    guaranteed_delivery: Option<UPSGuaranteedDelivery>,
}

#[derive(Deserialize)]
struct UPSService {
    #[serde(rename = "Code")]
    code: String,
}

#[derive(Deserialize)]
struct UPSTotalCharges {
    #[serde(rename = "MonetaryValue")]
    monetary_value: f32,
}

#[derive(Deserialize)]
struct UPSGuaranteedDelivery {
    #[serde(rename = "BusinessDaysInTransit")]
    business_days_in_transit: Option<u32>,
}

#[derive(Serialize)]
struct UPSLabelRequest {
    #[serde(rename = "ShipmentRequest")]
    shipment_request: UPSShipmentRequestDetails,
}

#[derive(Serialize)]
struct UPSShipmentRequestDetails {
    #[serde(rename = "Shipper")]
    shipper: UPSShipper,
    #[serde(rename = "ShipTo")]
    ship_to: UPSAddress,
    #[serde(rename = "Package")]
    package: UPSPackage,
    #[serde(rename = "Service")]
    service: UPSService,
    #[serde(rename = "LabelSpecification")]
    label_specification: UPSLabelSpecification,
}

#[derive(Serialize)]
struct UPSShipper {
    #[serde(rename = "AccountNumber")]
    account_number: String,
    #[serde(rename = "Address")]
    address: UPSAddress,
}

#[derive(Serialize)]
struct UPSLabelSpecification {
    #[serde(rename = "LabelPrintMethod")]
    label_print_method: UPSLabelPrintMethod,
}

#[derive(Serialize)]
struct UPSLabelPrintMethod {
    #[serde(rename = "Code")]
    code: String,
}

#[derive(Deserialize)]
struct UPSLabelResponse {
    #[serde(rename = "ShipmentResponse")]
    shipment_response: UPSShipmentResponseDetails,
}

#[derive(Deserialize)]
struct UPSShipmentResponseDetails {
    #[serde(rename = "ShipmentResults")]
    shipment_results: UPSShipmentResults,
}

#[derive(Deserialize)]
struct UPSShipmentResults {
    #[serde(rename = "TrackingNumber")]
    tracking_number: String,
    #[serde(rename = "Label")]
    label: UPSLabel,
}

#[derive(Deserialize)]
struct UPSLabel {
    #[serde(rename = "LabelURL")]
    label_url: String,
    #[serde(rename = "LabelExpiryDate")]
    label_expiry_date: Option<String>,
}

#[derive(Deserialize)]
struct UPSTrackResponse {
    #[serde(rename = "TrackResponse")]
    track_response: UPSTrackResponseDetails,
}

#[derive(Deserialize)]
struct UPSTrackResponseDetails {
    #[serde(rename = "Shipment")]
    shipment: UPSShipmentTrack,
}

#[derive(Deserialize)]
struct UPSShipmentTrack {
    #[serde(rename = "Package")]
    package: UPSTrackPackage,
}

#[derive(Deserialize)]
struct UPSTrackPackage {
    #[serde(rename = "Activity")]
    activity: Vec<UPSTrackActivity>,
    #[serde(rename = "Status")]
    status: UPSTrackStatus,
}

#[derive(Deserialize)]
struct UPSTrackStatus {
    #[serde(rename = "Description")]
    description: String,
}

#[derive(Deserialize)]
struct UPSTrackActivity {
    #[serde(rename = "ActivityLocation")]
    activity_location: UPSTrackLocation,
    #[serde(rename = "Status")]
    status: UPSTrackActivityStatus,
    #[serde(rename = "Date")]
    date: String,
}

#[derive(Deserialize)]
struct UPSTrackLocation {
    #[serde(rename = "Address")]
    address: UPSTrackLocationAddress,
}

#[derive(Deserialize)]
struct UPSTrackLocationAddress {
    #[serde(rename = "City")]
    city: String,
}

#[derive(Deserialize)]
struct UPSTrackActivityStatus {
    #[serde(rename = "Description")]
    description: String,
}

async fn get_oauth_token() -> Result<String, ShippingError> {
    let client_id = std::env::var("UPS_CLIENT_ID")
        .map_err(|_| ShippingError::EnvVarMissing("UPS_CLIENT_ID".to_string()))?;
    let client_secret = std::env::var("UPS_CLIENT_SECRET")
        .map_err(|_| ShippingError::EnvVarMissing("UPS_CLIENT_SECRET".to_string()))?;
    let url = std::env::var("URL_API_CARRIER_UPS")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_UPS".to_string()))?;
    let client = Client::new();

    let payload = UPSAuthRequest {
        grant_type: "client_credentials".to_string(),
        client_id,
        client_secret,
    };

    let operation = || async {
        client
            .post(format!("{}/security/v1/oauth/token", url))
            .form(&payload)
            .send()
            .await
            .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))
    };

    let res = retry(Fixed::from_millis(1000).take(3), operation)
        .await
        .map_err(|e| ShippingError::ApiRequestFailed(format!("Échec après retries : {}", e)))?;

    if !res.status().is_success() {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur OAuth UPS : {}", res.status())));
    }

    let data: UPSAuthResponse = res
        .json()
        .await
        .map_err(|e| ShippingError::JsonParsingError(e.to_string()))?;

    Ok(data.access_token)
}

pub async fn get_rates(shipment: &Shipment) -> Result<Vec<ShippingRate>, ShippingError> {
    let token = get_oauth_token().await?;
    let url = std::env::var("URL_API_CARRIER_UPS")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_UPS".to_string()))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;

    let payload = UPSRateRequest {
        rate_request: UPSRateRequestDetails {
            shipper: UPSAddress {
                postal_code: shipment.sender.postal_code.clone(),
                country_code: shipment.sender.country_code.clone(),
            },
            ship_to: UPSAddress {
                postal_code: shipment.recipient.postal_code.clone(),
                country_code: shipment.recipient.country_code.clone(),
            },
            package: UPSPackage {
                weight: UPSWeight {
                    unit: UPSUnit { code: "KGS".to_string() },
                    weight: shipment.package.weight_kg,
                },
                dimensions: UPSDimensions {
                    unit: UPSUnit { code: "CM".to_string() },
                    length: shipment.package.length_cm,
                    width: shipment.package.width_cm,
                    height: shipment.package.height_cm,
                },
            },
        },
    };

    let operation = || async {
        client
            .post(format!("{}/api/rating/v1/Shop", url))
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
        return Err(ShippingError::ApiRequestFailed(format!("Erreur API UPS : {}", res.status())));
    }

    let data: UPSRateResponse = res
        .json()
        .await
        .map_err(|e| ShippingError::JsonParsingError(e.to_string()))?;

    let rates = data.rate_response.rated_shipment.into_iter().map(|r| ShippingRate {
        service_name: r.service.code,
        price_eur: r.total_charges.monetary_value,
        estimated_days: r.guaranteed_delivery.and_then(|g| g.business_days_in_transit),
    }).collect();

    Ok(rates)
}

pub async fn create_label(shipment: &Shipment) -> Result<LabelResponse, ShippingError> {
    let token = get_oauth_token().await?;
    let url = std::env::var("URL_API_CARRIER_UPS")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_UPS".to_string()))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;

    let payload = UPSLabelRequest {
        shipment_request: UPSShipmentRequestDetails {
            shipper: UPSShipper {
                account_number: std::env::var("UPS_ACCOUNT_NUMBER")
                    .map_err(|_| ShippingError::EnvVarMissing("UPS_ACCOUNT_NUMBER".to_string()))?,
                address: UPSAddress {
                    postal_code: shipment.sender.postal_code.clone(),
                    country_code: shipment.sender.country_code.clone(),
                },
            },
            ship_to: UPSAddress {
                postal_code: shipment.recipient.postal_code.clone(),
                country_code: shipment.recipient.country_code.clone(),
            },
            package: UPSPackage {
                weight: UPSWeight {
                    unit: UPSUnit { code: "KGS".to_string() },
                    weight: shipment.package.weight_kg,
                },
                dimensions: UPSDimensions {
                    unit: UPSUnit { code: "CM".to_string() },
                    length: shipment.package.length_cm,
                    width: shipment.package.width_cm,
                    height: shipment.package.height_cm,
                },
            },
            service: UPSService { code: "03".to_string() }, // UPS Ground
            label_specification: UPSLabelSpecification {
                label_print_method: UPSLabelPrintMethod { code: "PDF".to_string() },
            },
        },
    };

    let operation = || async {
        client
            .post(format!("{}/api/shipments/v1/ship", url))
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
        return Err(ShippingError::ApiRequestFailed(format!("Erreur API UPS : {}", res.status())));
    }

    let data: UPSLabelResponse = res
        .json()
        .await
        .map_err(|e| ShippingError::JsonParsingError(e.to_string()))?;

    Ok(LabelResponse {
        label_url: data.shipment_response.shipment_results.label.label_url,
        tracking_number: data.shipment_response.shipment_results.tracking_number,
        expiry_date: data.shipment_response.shipment_results.label.label_expiry_date,
    })
}

pub async fn track_package(tracking_number: &str) -> Result<TrackingResponse, ShippingError> {
    let token = get_oauth_token().await?;
    let url = std::env::var("URL_API_CARRIER_UPS")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_UPS".to_string()))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;

    let operation = || async {
        client
            .get(format!("{}/api/track/v1/details/{}", url, tracking_number))
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))
    };

    let res = retry(Fixed::from_millis(1000).take(3), operation)
        .await
        .map_err(|e| ShippingError::ApiRequestFailed(format!("Échec après retries : {}", e)))?;

    if !res.status().is_success() {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur API UPS : {}", res.status())));
    }

    let data: UPSTrackResponse = res
        .json()
        .await
        .map_err(|e| ShippingError::JsonParsingError(e.to_string()))?;

    let package = &data.track_response.shipment.package;

    Ok(TrackingResponse {
        status: package.status.description.clone(),
        last_updated: package.activity.get(0).map_or("".to_string(), |a| a.date.clone()),
        events: package.activity.iter().map(|a| TrackingEvent {
            date: a.date.clone(),
            location: a.activity_location.address.city.clone(),
            description: a.status.description.clone(),
        }).collect(),
    })
}