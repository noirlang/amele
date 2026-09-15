//! Android cihazlarda dosya ağacı ve metadata çıkarma orkestratörü.
//! Sistemdeki tüm dosya ve dizinlerin izinlerini, erişim ve değiştirilme tarihlerini
//! MFT benzeri kronolojik bir tabloya dönüştürür.

mod bundle;
mod format;
mod outputs;
mod parsers;

pub use bundle::write_logical_mft_bundle;
pub use format::MftBundleInfo;
pub use outputs::write_logical_analysis_outputs;
