use crate::models::UserReview;
use crate::services::{ServiceError, ServiceResult};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Review service for managing user reviews and ratings business logic
pub struct ReviewService {
    /// In-memory review cache (in real app would use database)
    reviews: HashMap<String, UserReview>,
}

impl ReviewService {
    pub fn new() -> Self {
        Self {
            reviews: HashMap::new(),
        }
    }

    /// Submit a new review
    pub fn submit_review(
        &mut self,
        user_id: String,
        store_id: Option<String>,
        product_id: Option<String>,
        rating: i32,
        comment: String,
    ) -> ServiceResult<UserReview> {
        // Validate input
        self.validate_review_data(&rating, &comment, &store_id, &product_id)?;

        // Check if user already reviewed this item
        if let Some(existing) = self.find_existing_review(&user_id, &store_id, &product_id) {
            return Err(ServiceError::BusinessRuleViolation(format!(
                "User has already reviewed this item: {}",
                existing.id
            )));
        }

        // Create review
        let review = UserReview::new(user_id, store_id, product_id, rating, comment);

        // Store review
        self.reviews.insert(review.id.clone(), review.clone());

        log::info!(
            "Review submitted: {} stars by user {}",
            rating,
            review.user_id
        );
        Ok(review)
    }

    /// Get review by ID
    pub fn get_review(&self, review_id: &str) -> ServiceResult<UserReview> {
        self.reviews
            .get(review_id)
            .cloned()
            .ok_or_else(|| ServiceError::NotFound(format!("Review {} not found", review_id)))
    }

    /// Update a review
    pub fn update_review(
        &mut self,
        review_id: &str,
        user_id: &str,
        rating: Option<i32>,
        comment: Option<String>,
    ) -> ServiceResult<UserReview> {
        // Validate inputs first
        if let Some(new_rating) = rating {
            self.validate_rating(new_rating)?;
        }

        if let Some(ref new_comment) = comment {
            self.validate_comment(new_comment)?;
        }

        let review = self
            .reviews
            .get_mut(review_id)
            .ok_or_else(|| ServiceError::NotFound(format!("Review {} not found", review_id)))?;

        // Check permission
        if review.user_id != user_id {
            return Err(ServiceError::PermissionDenied(
                "Cannot update another user's review".to_string(),
            ));
        }

        // Update fields if provided
        if let Some(new_rating) = rating {
            review.rating = new_rating;
        }

        if let Some(new_comment) = comment {
            review.comment = new_comment;
        }

        log::info!("Review updated: {}", review_id);
        Ok(review.clone())
    }

    /// Delete a review
    pub fn delete_review(&mut self, review_id: &str, user_id: &str) -> ServiceResult<()> {
        let review = self
            .reviews
            .get(review_id)
            .ok_or_else(|| ServiceError::NotFound(format!("Review {} not found", review_id)))?;

        // Check permission
        if review.user_id != user_id {
            return Err(ServiceError::PermissionDenied(
                "Cannot delete another user's review".to_string(),
            ));
        }

        self.reviews.remove(review_id);

        log::info!("Review deleted: {}", review_id);
        Ok(())
    }

    /// Get reviews for a store
    pub fn get_store_reviews(&self, store_id: &str) -> ServiceResult<Vec<UserReview>> {
        let reviews: Vec<UserReview> = self
            .reviews
            .values()
            .filter(|r| r.store_id.as_ref() == Some(&store_id.to_string()))
            .cloned()
            .collect();

        Ok(reviews)
    }

    /// Get reviews for a product
    pub fn get_product_reviews(&self, product_id: &str) -> ServiceResult<Vec<UserReview>> {
        let reviews: Vec<UserReview> = self
            .reviews
            .values()
            .filter(|r| r.product_id.as_ref() == Some(&product_id.to_string()))
            .cloned()
            .collect();

        Ok(reviews)
    }

    /// Get reviews by a user
    pub fn get_user_reviews(&self, user_id: &str) -> ServiceResult<Vec<UserReview>> {
        let reviews: Vec<UserReview> = self
            .reviews
            .values()
            .filter(|r| r.user_id == user_id)
            .cloned()
            .collect();

        Ok(reviews)
    }

    /// Calculate average rating for a store
    pub fn get_store_average_rating(&self, store_id: &str) -> ServiceResult<f64> {
        let store_reviews = self.get_store_reviews(store_id)?;

        if store_reviews.is_empty() {
            return Ok(0.0);
        }

        let total_rating: i32 = store_reviews.iter().map(|r| r.rating).sum();
        let average = total_rating as f64 / store_reviews.len() as f64;

        Ok(average)
    }

    /// Calculate average rating for a product
    pub fn get_product_average_rating(&self, product_id: &str) -> ServiceResult<f64> {
        let product_reviews = self.get_product_reviews(product_id)?;

        if product_reviews.is_empty() {
            return Ok(0.0);
        }

        let total_rating: i32 = product_reviews.iter().map(|r| r.rating).sum();
        let average = total_rating as f64 / product_reviews.len() as f64;

        Ok(average)
    }

    /// Get rating distribution for a store
    pub fn get_store_rating_distribution(
        &self,
        store_id: &str,
    ) -> ServiceResult<RatingDistribution> {
        let store_reviews = self.get_store_reviews(store_id)?;
        self.calculate_rating_distribution(store_reviews)
    }

    /// Get rating distribution for a product
    pub fn get_product_rating_distribution(
        &self,
        product_id: &str,
    ) -> ServiceResult<RatingDistribution> {
        let product_reviews = self.get_product_reviews(product_id)?;
        self.calculate_rating_distribution(product_reviews)
    }

    /// Get recent reviews with pagination
    pub fn get_recent_reviews(
        &self,
        offset: usize,
        limit: usize,
    ) -> ServiceResult<Vec<UserReview>> {
        let mut all_reviews: Vec<UserReview> = self.reviews.values().cloned().collect();

        // Sort by creation date (newest first)
        all_reviews.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let reviews: Vec<UserReview> = all_reviews.into_iter().skip(offset).take(limit).collect();

        Ok(reviews)
    }

    /// Search reviews by comment content
    pub fn search_reviews(&self, query: &str) -> ServiceResult<Vec<UserReview>> {
        let query_lower = query.to_lowercase();

        let reviews: Vec<UserReview> = self
            .reviews
            .values()
            .filter(|r| r.comment.to_lowercase().contains(&query_lower))
            .cloned()
            .collect();

        Ok(reviews)
    }

    /// Get reviews within a rating range
    pub fn get_reviews_by_rating_range(
        &self,
        min_rating: i32,
        max_rating: i32,
    ) -> ServiceResult<Vec<UserReview>> {
        if min_rating < 1 || max_rating > 5 || min_rating > max_rating {
            return Err(ServiceError::ValidationError(
                "Invalid rating range".to_string(),
            ));
        }

        let reviews: Vec<UserReview> = self
            .reviews
            .values()
            .filter(|r| r.rating >= min_rating && r.rating <= max_rating)
            .cloned()
            .collect();

        Ok(reviews)
    }

    /// Get review statistics
    pub fn get_review_stats(&self) -> ServiceResult<ReviewStats> {
        let total_reviews = self.reviews.len();

        let store_reviews = self
            .reviews
            .values()
            .filter(|r| r.store_id.is_some())
            .count();

        let product_reviews = self
            .reviews
            .values()
            .filter(|r| r.product_id.is_some())
            .count();

        let avg_rating = if total_reviews > 0 {
            let total_rating: i32 = self.reviews.values().map(|r| r.rating).sum();
            total_rating as f64 / total_reviews as f64
        } else {
            0.0
        };

        let unique_users: std::collections::HashSet<String> =
            self.reviews.values().map(|r| r.user_id.clone()).collect();

        let rating_distribution =
            self.calculate_rating_distribution(self.reviews.values().cloned().collect())?;

        Ok(ReviewStats {
            total_reviews,
            store_reviews,
            product_reviews,
            average_rating: avg_rating,
            unique_reviewers: unique_users.len(),
            rating_distribution,
        })
    }

    /// Get top reviewed items
    pub fn get_top_reviewed_items(&self, limit: usize) -> ServiceResult<TopReviewedItems> {
        // Group by store
        let mut store_counts: HashMap<String, usize> = HashMap::new();
        for review in self.reviews.values() {
            if let Some(ref store_id) = review.store_id {
                *store_counts.entry(store_id.clone()).or_insert(0) += 1;
            }
        }

        // Group by product
        let mut product_counts: HashMap<String, usize> = HashMap::new();
        for review in self.reviews.values() {
            if let Some(ref product_id) = review.product_id {
                *product_counts.entry(product_id.clone()).or_insert(0) += 1;
            }
        }

        // Sort and limit
        let mut top_stores: Vec<(String, usize)> = store_counts.into_iter().collect();
        top_stores.sort_by(|a, b| b.1.cmp(&a.1));
        top_stores.truncate(limit);

        let mut top_products: Vec<(String, usize)> = product_counts.into_iter().collect();
        top_products.sort_by(|a, b| b.1.cmp(&a.1));
        top_products.truncate(limit);

        Ok(TopReviewedItems {
            stores: top_stores,
            products: top_products,
        })
    }

    // Helper methods

    fn validate_review_data(
        &self,
        rating: &i32,
        comment: &str,
        store_id: &Option<String>,
        product_id: &Option<String>,
    ) -> ServiceResult<()> {
        // Must review either a store or a product, not both or neither
        match (store_id, product_id) {
            (Some(_), Some(_)) => {
                return Err(ServiceError::ValidationError(
                    "Cannot review both store and product in one review".to_string(),
                ));
            }
            (None, None) => {
                return Err(ServiceError::ValidationError(
                    "Must review either a store or a product".to_string(),
                ));
            }
            _ => {}
        }

        self.validate_rating(*rating)?;
        self.validate_comment(comment)?;

        Ok(())
    }

    fn validate_rating(&self, rating: i32) -> ServiceResult<()> {
        if rating < 1 || rating > 5 {
            return Err(ServiceError::ValidationError(
                "Rating must be between 1 and 5".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_comment(&self, comment: &str) -> ServiceResult<()> {
        if comment.trim().is_empty() {
            return Err(ServiceError::ValidationError(
                "Comment cannot be empty".to_string(),
            ));
        }

        if comment.len() > 1000 {
            return Err(ServiceError::ValidationError(
                "Comment too long".to_string(),
            ));
        }

        Ok(())
    }

    fn find_existing_review(
        &self,
        user_id: &str,
        store_id: &Option<String>,
        product_id: &Option<String>,
    ) -> Option<&UserReview> {
        self.reviews.values().find(|r| {
            r.user_id == user_id && r.store_id == *store_id && r.product_id == *product_id
        })
    }

    fn calculate_rating_distribution(
        &self,
        reviews: Vec<UserReview>,
    ) -> ServiceResult<RatingDistribution> {
        let mut distribution = [0; 5];

        for review in &reviews {
            let index = (review.rating - 1) as usize;
            if index < 5 {
                distribution[index] += 1;
            }
        }

        Ok(RatingDistribution {
            one_star: distribution[0],
            two_star: distribution[1],
            three_star: distribution[2],
            four_star: distribution[3],
            five_star: distribution[4],
            total: reviews.len(),
        })
    }
}

impl Default for ReviewService {
    fn default() -> Self {
        Self::new()
    }
}

/// Rating distribution statistics
#[derive(Debug, Clone)]
pub struct RatingDistribution {
    pub one_star: usize,
    pub two_star: usize,
    pub three_star: usize,
    pub four_star: usize,
    pub five_star: usize,
    pub total: usize,
}

/// Review statistics
#[derive(Debug, Clone)]
pub struct ReviewStats {
    pub total_reviews: usize,
    pub store_reviews: usize,
    pub product_reviews: usize,
    pub average_rating: f64,
    pub unique_reviewers: usize,
    pub rating_distribution: RatingDistribution,
}

/// Top reviewed items
#[derive(Debug, Clone)]
pub struct TopReviewedItems {
    pub stores: Vec<(String, usize)>,
    pub products: Vec<(String, usize)>,
}
