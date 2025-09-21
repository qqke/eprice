use crate::services::ServiceResult;
use crate::services::price_service::PriceService;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Verification manager for handling price record verification
pub struct VerificationManager {
    // Store verification status and metadata
    verification_history: HashMap<String, VerificationRecord>,
}

#[derive(Debug, Clone)]
pub struct VerificationRecord {
    pub price_record_id: String,
    pub original_status: String,
    pub new_status: String,
    pub verified_by: String,
    pub timestamp: DateTime<Utc>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VerificationStats {
    pub total_pending: usize,
    pub total_verified: usize,
    pub total_rejected: usize,
    pub verification_rate: f64,      // Percentage of verified records
    pub recent_verifications: usize, // Verifications in last 24h
}

impl VerificationManager {
    pub fn new() -> Self {
        Self {
            verification_history: HashMap::new(),
        }
    }

    /// Verify a price record
    pub fn verify_price_record(
        &mut self,
        price_service: &mut PriceService,
        price_record_id: &str,
        verified_by: &str,
        reason: Option<String>,
    ) -> ServiceResult<()> {
        // Get the current record to store its status
        let current_record = price_service.get_price_record(price_record_id)?;
        let original_status = current_record.verification_status.clone();

        // Verify the record through the price service
        price_service.verify_price(price_record_id, true)?;

        // Record the verification action
        let verification_record = VerificationRecord {
            price_record_id: price_record_id.to_string(),
            original_status,
            new_status: "verified".to_string(),
            verified_by: verified_by.to_string(),
            timestamp: Utc::now(),
            reason,
        };

        self.verification_history
            .insert(price_record_id.to_string(), verification_record);

        Ok(())
    }

    /// Reject a price record
    pub fn reject_price_record(
        &mut self,
        price_service: &mut PriceService,
        price_record_id: &str,
        verified_by: &str,
        reason: Option<String>,
    ) -> ServiceResult<()> {
        // Get the current record to store its status
        let current_record = price_service.get_price_record(price_record_id)?;
        let original_status = current_record.verification_status.clone();

        // Reject the record through the price service
        price_service.verify_price(price_record_id, false)?;

        // Record the verification action
        let verification_record = VerificationRecord {
            price_record_id: price_record_id.to_string(),
            original_status,
            new_status: "rejected".to_string(),
            verified_by: verified_by.to_string(),
            timestamp: Utc::now(),
            reason,
        };

        self.verification_history
            .insert(price_record_id.to_string(), verification_record);

        Ok(())
    }

    /// Reset a price record to pending status
    pub fn reset_to_pending(
        &mut self,
        price_service: &mut PriceService,
        price_record_id: &str,
        verified_by: &str,
        reason: Option<String>,
    ) -> ServiceResult<()> {
        // Get the current record to store its status
        let current_record = price_service.get_price_record(price_record_id)?;
        let original_status = current_record.verification_status.clone();

        // Reset to pending status (this requires adding a method to price service)
        price_service.reset_price_record_status(price_record_id)?;

        // Record the verification action
        let verification_record = VerificationRecord {
            price_record_id: price_record_id.to_string(),
            original_status,
            new_status: "pending".to_string(),
            verified_by: verified_by.to_string(),
            timestamp: Utc::now(),
            reason,
        };

        self.verification_history
            .insert(price_record_id.to_string(), verification_record);

        Ok(())
    }

    /// Get verification statistics
    pub fn get_verification_stats(
        &self,
        price_service: &PriceService,
    ) -> ServiceResult<VerificationStats> {
        let submission_stats = price_service.get_submission_stats()?;

        let total_records = submission_stats.total_submissions;
        let verification_rate = if total_records > 0 {
            (submission_stats.verified_count as f64 / total_records as f64) * 100.0
        } else {
            0.0
        };

        // Count recent verifications (last 24h)
        let recent_cutoff = Utc::now() - chrono::Duration::hours(24);
        let recent_verifications = self
            .verification_history
            .values()
            .filter(|v| v.timestamp > recent_cutoff)
            .count();

        Ok(VerificationStats {
            total_pending: submission_stats.pending_count,
            total_verified: submission_stats.verified_count,
            total_rejected: submission_stats.rejected_count,
            verification_rate,
            recent_verifications,
        })
    }

    /// Get verification history for a specific price record
    pub fn get_verification_history(&self, price_record_id: &str) -> Option<&VerificationRecord> {
        self.verification_history.get(price_record_id)
    }

    /// Get all verification history
    pub fn get_all_verification_history(&self) -> Vec<&VerificationRecord> {
        self.verification_history.values().collect()
    }

    /// Bulk verify multiple price records
    pub fn bulk_verify_records(
        &mut self,
        price_service: &mut PriceService,
        price_record_ids: &[String],
        verified_by: &str,
        reason: Option<String>,
    ) -> ServiceResult<usize> {
        let mut success_count = 0;

        for record_id in price_record_ids {
            if self
                .verify_price_record(price_service, record_id, verified_by, reason.clone())
                .is_ok()
            {
                success_count += 1;
            }
        }

        Ok(success_count)
    }

    /// Bulk reject multiple price records
    pub fn bulk_reject_records(
        &mut self,
        price_service: &mut PriceService,
        price_record_ids: &[String],
        verified_by: &str,
        reason: Option<String>,
    ) -> ServiceResult<usize> {
        let mut success_count = 0;

        for record_id in price_record_ids {
            if self
                .reject_price_record(price_service, record_id, verified_by, reason.clone())
                .is_ok()
            {
                success_count += 1;
            }
        }

        Ok(success_count)
    }
}

impl Default for VerificationManager {
    fn default() -> Self {
        Self::new()
    }
}
