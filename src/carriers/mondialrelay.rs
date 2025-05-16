use crate::types::{Shipment, ShippingRate, LabelResponse, TrackingResponse, TrackingEvent};
use crate::error::ShippingError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use retry::{delay::Fixed, retry};
use std::time::Duration;
use md5::{Md5, Digest};

#[derive(Serialize)]
struct MondialRelaySoapRequest {
    #[serde(rename = "soap:Envelope")]
    envelope: SoapEnvelope,
}

#[derive(Serialize)]
struct SoapEnvelope {
    #[serde(rename = "xmlns:soap")]
    soap: String,
    #[serde(rename = "xmlns:ns1")]
    ns1: String,
    #[serde(rename = "soap:Body")]
    body: SoapBody,
}

#[derive(Serialize)]
struct SoapBody {
    #[serde(flatten)]
    operation: serde_json::Value,
}

#[derive(Serialize)]
struct MondialRelayRateRequest {
    #[serde(rename = "Enseigne")]
    enseigne: String,
    #[serde(rename = "Poids")]
    poids: String,
    #[serde(rename = "CodePostalDepart")]
    code_postal_depart: String,
    #[serde(rename = "CodePostalArrivee")]
    code_postal_arrivee: String,
    #[serde(rename = "PaysDepart")]
    pays_depart: String,
    #[serde(rename = "PaysArrivee")]
    pays_arrivee: String,
    #[serde(rename = "Security")]
    security: String,
}

#[derive(Deserialize)]
struct MondialRelayRateResponse {
    #[serde(rename = "WSI4_GetTarifResult")]
    result: MondialRelayRateResult,
}

#[derive(Deserialize)]
struct MondialRelayRateResult {
    #[serde(rename = "STAT")]
    stat: String,
    #[serde(rename = "Tarif")]
    tarif: Option<f32>,
}

#[derive(Serialize)]
struct MondialRelayLabelRequest {
    #[serde(rename = "Enseigne")]
    enseigne: String,
    #[serde(rename = "Poids")]
    poids: String,
    #[serde(rename = "Longueur")]
    longueur: String,
    #[serde(rename = "Largeur")]
    largeur: String,
    #[serde(rename = "Hauteur")]
    hauteur: String,
    #[serde(rename = "CodePostalExpediteur")]
    code_postal_expediteur: String,
    #[serde(rename = "PaysExpediteur")]
    pays_expediteur: String,
    #[serde(rename = "CodePostalDestinataire")]
    code_postal_destinataire: String,
    #[serde(rename = "PaysDestinataire")]
    pays_destinataire: String,
    #[serde(rename = "TypeColis")]
    type_colis: String,
    #[serde(rename = "Security")]
    security: String,
}

#[derive(Deserialize)]
struct MondialRelayLabelResponse {
    #[serde(rename = "WSI2_CreationEtiquetteResult")]
    result: MondialRelayLabelResult,
}

#[derive(Deserialize)]
struct MondialRelayLabelResult {
    #[serde(rename = "STAT")]
    stat: String,
    #[serde(rename = "Num")]
    num: Option<String>,
    #[serde(rename = "URL_Etiquette")]
    url_etiquette: Option<String>,
}

#[derive(Serialize)]
struct MondialRelayTrackRequest {
    #[serde(rename = "Enseigne")]
    enseigne: String,
    #[serde(rename = "Num")]
    num: String,
    #[serde(rename = "Security")]
    security: String,
}

#[derive(Deserialize)]
struct MondialRelayTrackResponse {
    #[serde(rename = "WSI2_TracingColisResult")]
    result: MondialRelayTrackResult,
}

#[derive(Deserialize)]
struct MondialRelayTrackResult {
    #[serde(rename = "STAT")]
    stat: String,
    #[serde(rename = "Statut")]
    statut: Option<String>,
    #[serde(rename = "DateDernierEvent")]
    date_dernier_event: Option<String>,
    #[serde(rename = "Evenements")]
    evenements: Option<Vec<MondialRelayEvent>>,
}

#[derive(Deserialize)]
struct MondialRelayEvent {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Lieu")]
    lieu: String,
    #[serde(rename = "Description")]
    description: String,
}

fn generate_security_hash(params: &str, private_key: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(params.to_string() + private_key);
    let result = hasher.finalize();
    format!("{:X}", result).to_uppercase()
}

pub async fn get_rates(shipment: &Shipment) -> Result<Vec<ShippingRate>, ShippingError> {
    let enseigne = std::env::var("MONDIALRELAY_BRAND_CODE")
        .map_err(|_| ShippingError::EnvVarMissing("MONDIALRELAY_BRAND_CODE".to_string()))?;
    let private_key = std::env::var("MONDIALRELAY_PRIVATE_KEY")
        .map_err(|_| ShippingError::EnvVarMissing("MONDIALRELAY_PRIVATE_KEY".to_string()))?;
    let url = std::env::var("URL_API_CARRIER_MONDIALRELAY")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_MONDIALRELAY".to_string()))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;

    let params = format!(
        "{}{:.0}{}{}",
        enseigne,
        shipment.package.weight_kg * 1000.0, // Convertir en grammes
        shipment.sender.postal_code,
        shipment.recipient.postal_code
    );
    let security = generate_security_hash(&params, &private_key);

    let payload = MondialRelaySoapRequest {
        envelope: SoapEnvelope {
            soap: "http://schemas.xmlsoap.org/soap/envelope/".to_string(),
            ns1: "http://www.mondialrelay.fr/webservice/".to_string(),
            body: SoapBody {
                operation: serde_json::json!({
                    "ns1:WSI4_GetTarif": {
                        "Enseigne": enseigne,
                        "Poids": format!("{:.0}", shipment.package.weight_kg * 1000.0),
                        "CodePostalDepart": shipment.sender.postal_code,
                        "CodePostalArrivee": shipment.recipient.postal_code,
                        "PaysDepart": shipment.sender.country_code,
                        "PaysArrivee": shipment.recipient.country_code,
                        "Security": security
                    }
                }),
            },
        },
    };

    let operation = || async {
        client
            .post(format!("{}/Web_Services.asmx", url))
            .header("Content-Type", "text/xml; charset=utf-8")
            .body(serde_xml_rs::to_string(&payload).map_err(|e| ShippingError::XmlParsingError(e.to_string()))?)
            .send()
            .await
            .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))
    };

    let res = retry(Fixed::from_millis(1000).take(3), operation)
        .await
        .map_err(|e| ShippingError::ApiRequestFailed(format!("Échec après retries : {}", e)))?;

    if !res.status().is_success() {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur API Mondial Relay : {}", res.status())));
    }

    let body = res
        .text()
        .await
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;
    let data: MondialRelayRateResponse = serde_xml_rs::from_str(&body)
        .map_err(|e| ShippingError::XmlParsingError(e.to_string()))?;

    if data.result.stat != "0" {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur Mondial Relay : STAT {}", data.result.stat)));
    }

    let rates = data.result.tarif.map(|tarif| vec![ShippingRate {
        service_name: "Point Relais".to_string(),
        price_eur: tarif,
        estimated_days: Some(3), // Estimation standard (3-5 jours)
    }]).unwrap_or_default();

    Ok(rates)
}

pub async fn create_label(shipment: &Shipment) -> Result<LabelResponse, ShippingError> {
    let enseigne = std::env::var("MONDIALRELAY_BRAND_CODE")
        .map_err(|_| ShippingError::EnvVarMissing("MONDIALRELAY_BRAND_CODE".to_string()))?;
    let private_key = std::env::var("MONDIALRELAY_PRIVATE_KEY")
        .map_err(|_| ShippingError::EnvVarMissing("MONDIALRELAY_PRIVATE_KEY".to_string()))?;
    let url = std::env::var("URL_API_CARRIER_MONDIALRELAY")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_MONDIALRELAY".to_string()))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;

    let params = format!(
        "{}{:.0}{}{}{}",
        enseigne,
        shipment.package.weight_kg * 1000.0,
        shipment.sender.postal_code,
        shipment.recipient.postal_code,
        "24R" // Type de colis standard
    );
    let security = generate_security_hash(&params, &private_key);

    let payload = MondialRelaySoapRequest {
        envelope: SoapEnvelope {
            soap: "http://schemas.xmlsoap.org/soap/envelope/".to_string(),
            ns1: "http://www.mondialrelay.fr/webservice/".to_string(),
            body: SoapBody {
                operation: serde_json::json!({
                    "ns1:WSI2_CreationEtiquette": {
                        "Enseigne": enseigne,
                        "Poids": format!("{:.0}", shipment.package.weight_kg * 1000.0),
                        "Longueur": format!("{:.0}", shipment.package.length_cm),
                        "Largeur": format!("{:.0}", shipment.package.width_cm),
                        "Hauteur": format!("{:.0}", shipment.package.height_cm),
                        "CodePostalExpediteur": shipment.sender.postal_code,
                        "PaysExpediteur": shipment.sender.country_code,
                        "CodePostalDestinataire": shipment.recipient.postal_code,
                        "PaysDestinataire": shipment.recipient.country_code,
                        "TypeColis": "24R",
                        "Security": security
                    }
                }),
            },
        },
    };

    let operation = || async {
        client
            .post(format!("{}/Web_Services.asmx", url))
            .header("Content-Type", "text/xml; charset=utf-8")
            .body(serde_xml_rs::to_string(&payload).map_err(|e| ShippingError::XmlParsingError(e.to_string()))?)
            .send()
            .await
            .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))
    };

    let res = retry(Fixed::from_millis(1000).take(3), operation)
        .await
        .map_err(|e| ShippingError::ApiRequestFailed(format!("Échec après retries : {}", e)))?;

    if !res.status().is_success() {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur API Mondial Relay : {}", res.status())));
    }

    let body = res
        .text()
        .await
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;
    let data: MondialRelayLabelResponse = serde_xml_rs::from_str(&body)
        .map_err(|e| ShippingError::XmlParsingError(e.to_string()))?;

    if data.result.stat != "0" {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur Mondial Relay : STAT {}", data.result.stat)));
    }

    Ok(LabelResponse {
        label_url: data.result.url_etiquette.ok_or_else(|| ShippingError::ApiRequestFailed("URL d'étiquette manquante".to_string()))?,
        tracking_number: data.result.num.ok_or_else(|| ShippingError::ApiRequestFailed("Numéro de suivi manquant".to_string()))?,
        expiry_date: None, // Mondial Relay ne fournit pas de date d'expiration
    })
}

pub async fn track_package(tracking_number: &str) -> Result<TrackingResponse, ShippingError> {
    let enseigne = std::env::var("MONDIALRELAY_BRAND_CODE")
        .map_err(|_| ShippingError::EnvVarMissing("MONDIALRELAY_BRAND_CODE".to_string()))?;
    let private_key = std::env::var("MONDIALRELAY_PRIVATE_KEY")
        .map_err(|_| ShippingError::EnvVarMissing("MONDIALRELAY_PRIVATE_KEY".to_string()))?;
    let url = std::env::var("URL_API_CARRIER_MONDIALRELAY")
        .map_err(|_| ShippingError::EnvVarMissing("URL_API_CARRIER_MONDIALRELAY".to_string()))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;

    let params = format!("{}{}", enseigne, tracking_number);
    let security = generate_security_hash(&params, &private_key);

    let payload = MondialRelaySoapRequest {
        envelope: SoapEnvelope {
            soap: "http://schemas.xmlsoap.org/soap/envelope/".to_string(),
            ns1: "http://www.mondialrelay.fr/webservice/".to_string(),
            body: SoapBody {
                operation: serde_json::json!({
                    "ns1:WSI2_TracingColis": {
                        "Enseigne": enseigne,
                        "Num": tracking_number,
                        "Security": security
                    }
                }),
            },
        },
    };

    let operation = || async {
        client
            .post(format!("{}/Web_Services.asmx", url))
            .header("Content-Type", "text/xml; charset=utf-8")
            .body(serde_xml_rs::to_string(&payload).map_err(|e| ShippingError::XmlParsingError(e.to_string()))?)
            .send()
            .await
            .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))
    };

    let res = retry(Fixed::from_millis(1000).take(3), operation)
        .await
        .map_err(|e| ShippingError::ApiRequestFailed(format!("Échec après retries : {}", e)))?;

    if !res.status().is_success() {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur API Mondial Relay : {}", res.status())));
    }

    let body = res
        .text()
        .await
        .map_err(|e| ShippingError::ApiRequestFailed(e.to_string()))?;
    let data: MondialRelayTrackResponse = serde_xml_rs::from_str(&body)
        .map_err(|e| ShippingError::XmlParsingError(e.to_string()))?;

    if data.result.stat != "0" {
        return Err(ShippingError::ApiRequestFailed(format!("Erreur Mondial Relay : STAT {}", data.result.stat)));
    }

    Ok(TrackingResponse {
        status: data.result.statut.unwrap_or_default(),
        last_updated: data.result.date_dernier_event.unwrap_or_default(),
        events: data.result.evenements.unwrap_or_default().into_iter().map(|e| TrackingEvent {
            date: e.date,
            location: e.lieu,
            description: e.description,
        }).collect(),
    })
}