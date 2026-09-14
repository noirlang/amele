//! ============================================================================
//! # ANDROİD MFT (MASTER FILE TABLE) ANALİZ MOTORU (src/android_mft.rs)
//! ============================================================================
//!
//! 1. Android MFT Konsepti: Android dosya kayıtlarını MFT formatında yapılandırıp arama ve analize açar.
//! ============================================================================
mod bundle;
mod format;
mod outputs;
mod parsers;

pub use bundle::write_logical_mft_bundle;
pub use format::MftBundleInfo;
pub use outputs::write_logical_analysis_outputs;
