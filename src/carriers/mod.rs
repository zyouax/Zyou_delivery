/// Module pour Colissimo
#[cfg(feature = "colissimo")]
pub mod colissimo;

/// Module pour Chronopost
#[cfg(feature = "chronopost")]
pub mod chronopost;

/// Module pour FedEx
#[cfg(feature = "fedex")]
pub mod fedex;

/// Module pour UPS
#[cfg(feature = "ups")]
pub mod ups;

/// Module pour DHL
#[cfg(feature = "dhl")]
pub mod dhl;
