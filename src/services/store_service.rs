use crate::models::Store;
use crate::services::{ServiceError, ServiceResult};
use std::collections::HashMap;

/// Store service for managing store operations and business logic
pub struct StoreService {
    /// In-memory store cache (in real app would use database)
    stores: HashMap<String, Store>,
}

impl StoreService {
    pub fn new() -> Self {
        let mut service = Self {
            stores: HashMap::new(),
        };

        // Initialize with sample stores
        service.init_sample_stores();
        service
    }

    /// Create a new store
    pub fn create_store(
        &mut self,
        name: String,
        address: String,
        latitude: f64,
        longitude: f64,
        opening_hours: String,
        phone: String,
        tags: Vec<String>,
        symbol: char,
    ) -> ServiceResult<Store> {
        // Validate input
        self.validate_store_data(&name, &address, latitude, longitude, &phone)?;

        // Create store
        let store = Store::new(
            name,
            address,
            latitude,
            longitude,
            opening_hours,
            phone,
            tags,
            symbol,
        );

        // Store it
        self.stores.insert(store.id.clone(), store.clone());

        log::info!("Store created: {}", store.name);
        Ok(store)
    }

    /// Get store by ID
    pub fn get_store(&self, store_id: &str) -> ServiceResult<Store> {
        self.stores
            .get(store_id)
            .cloned()
            .ok_or_else(|| ServiceError::NotFound(format!("Store {} not found", store_id)))
    }

    /// Update store information
    pub fn update_store(
        &mut self,
        store_id: &str,
        name: Option<String>,
        address: Option<String>,
        latitude: Option<f64>,
        longitude: Option<f64>,
        opening_hours: Option<String>,
        phone: Option<String>,
        tags: Option<Vec<String>>,
    ) -> ServiceResult<Store> {
        // Validate inputs first
        if let Some(ref new_name) = name {
            self.validate_store_name(new_name)?;
        }

        if let Some(ref new_address) = address {
            self.validate_address(new_address)?;
        }

        if let Some(ref new_phone) = phone {
            self.validate_phone(new_phone)?;
        }

        // Get store for coordinate validation
        let existing_store = self
            .stores
            .get(store_id)
            .ok_or_else(|| ServiceError::NotFound(format!("Store {} not found", store_id)))?;

        if let Some(lat) = latitude {
            self.validate_coordinates(lat, existing_store.longitude)?;
        }

        if let Some(lng) = longitude {
            self.validate_coordinates(existing_store.latitude, lng)?;
        }

        // Now get mutable reference
        let store = self.stores.get_mut(store_id).unwrap(); // Safe since we checked above

        // Update fields if provided
        if let Some(new_name) = name {
            store.name = new_name;
        }

        if let Some(new_address) = address {
            store.address = new_address;
        }

        if let Some(lat) = latitude {
            store.latitude = lat;
        }

        if let Some(lng) = longitude {
            store.longitude = lng;
        }

        if let Some(new_hours) = opening_hours {
            store.opening_hours = new_hours;
        }

        if let Some(new_phone) = phone {
            store.phone = new_phone;
        }

        if let Some(new_tags) = tags {
            store.tags = new_tags;
        }

        log::info!("Store updated: {}", store.name);
        Ok(store.clone())
    }

    /// Delete store
    pub fn delete_store(&mut self, store_id: &str) -> ServiceResult<()> {
        let store = self
            .stores
            .remove(store_id)
            .ok_or_else(|| ServiceError::NotFound(format!("Store {} not found", store_id)))?;

        log::info!("Store deleted: {}", store.name);
        Ok(())
    }

    /// Search stores by name or address
    pub fn search_stores(&self, query: &str) -> ServiceResult<Vec<Store>> {
        let query_lower = query.to_lowercase();

        let stores: Vec<Store> = self
            .stores
            .values()
            .filter(|s| {
                s.name.to_lowercase().contains(&query_lower)
                    || s.address.to_lowercase().contains(&query_lower)
                    || s.tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect();

        Ok(stores)
    }

    /// Find stores near a location
    pub fn find_stores_near(
        &self,
        latitude: f64,
        longitude: f64,
        radius_km: f64,
    ) -> ServiceResult<Vec<StoreDistance>> {
        let mut store_distances: Vec<StoreDistance> = self
            .stores
            .values()
            .map(|store| {
                let distance = store.distance_to(latitude, longitude);
                StoreDistance {
                    store: store.clone(),
                    distance_km: distance,
                }
            })
            .filter(|sd| sd.distance_km <= radius_km)
            .collect();

        // Sort by distance
        store_distances.sort_by(|a, b| {
            a.distance_km
                .partial_cmp(&b.distance_km)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(store_distances)
    }

    /// Get stores by tag
    pub fn get_stores_by_tag(&self, tag: &str) -> ServiceResult<Vec<Store>> {
        let stores: Vec<Store> = self
            .stores
            .values()
            .filter(|s| s.tags.contains(&tag.to_string()))
            .cloned()
            .collect();

        Ok(stores)
    }

    /// Get all stores with pagination
    pub fn list_stores(&self, offset: usize, limit: usize) -> ServiceResult<Vec<Store>> {
        let stores: Vec<Store> = self
            .stores
            .values()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();

        Ok(stores)
    }

    /// Update store rating
    pub fn update_store_rating(&mut self, store_id: &str, new_rating: f64) -> ServiceResult<f64> {
        let store = self
            .stores
            .get_mut(store_id)
            .ok_or_else(|| ServiceError::NotFound(format!("Store {} not found", store_id)))?;

        if new_rating < 0.0 || new_rating > 5.0 {
            return Err(ServiceError::ValidationError(
                "Rating must be between 0 and 5".to_string(),
            ));
        }

        store.rating = new_rating;

        log::info!("Store rating updated: {} -> {:.1}", store.name, new_rating);
        Ok(store.rating)
    }

    /// Get store statistics
    pub fn get_store_stats(&self) -> ServiceResult<StoreStats> {
        let total_stores = self.stores.len();

        let avg_rating = if total_stores > 0 {
            self.stores.values().map(|s| s.rating).sum::<f64>() / total_stores as f64
        } else {
            0.0
        };

        let tag_counts: HashMap<String, usize> =
            self.stores
                .values()
                .flat_map(|s| &s.tags)
                .fold(HashMap::new(), |mut acc, tag| {
                    *acc.entry(tag.clone()).or_insert(0) += 1;
                    acc
                });

        let stores_by_rating = self.categorize_stores_by_rating();

        Ok(StoreStats {
            total_stores,
            average_rating: avg_rating,
            tag_counts,
            stores_by_rating,
        })
    }

    /// Get stores within a bounding box
    pub fn get_stores_in_bounds(
        &self,
        north: f64,
        south: f64,
        east: f64,
        west: f64,
    ) -> ServiceResult<Vec<Store>> {
        let stores: Vec<Store> = self
            .stores
            .values()
            .filter(|s| {
                s.latitude >= south
                    && s.latitude <= north
                    && s.longitude >= west
                    && s.longitude <= east
            })
            .cloned()
            .collect();

        Ok(stores)
    }

    // Helper methods

    fn validate_store_data(
        &self,
        name: &str,
        address: &str,
        latitude: f64,
        longitude: f64,
        phone: &str,
    ) -> ServiceResult<()> {
        self.validate_store_name(name)?;
        self.validate_address(address)?;
        self.validate_coordinates(latitude, longitude)?;
        self.validate_phone(phone)?;
        Ok(())
    }

    fn validate_store_name(&self, name: &str) -> ServiceResult<()> {
        if name.trim().is_empty() {
            return Err(ServiceError::ValidationError(
                "Store name cannot be empty".to_string(),
            ));
        }

        if name.len() > 200 {
            return Err(ServiceError::ValidationError(
                "Store name too long".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_address(&self, address: &str) -> ServiceResult<()> {
        if address.trim().is_empty() {
            return Err(ServiceError::ValidationError(
                "Address cannot be empty".to_string(),
            ));
        }

        if address.len() > 500 {
            return Err(ServiceError::ValidationError(
                "Address too long".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_coordinates(&self, latitude: f64, longitude: f64) -> ServiceResult<()> {
        if latitude < -90.0 || latitude > 90.0 {
            return Err(ServiceError::ValidationError(
                "Invalid latitude".to_string(),
            ));
        }

        if longitude < -180.0 || longitude > 180.0 {
            return Err(ServiceError::ValidationError(
                "Invalid longitude".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_phone(&self, phone: &str) -> ServiceResult<()> {
        if phone.trim().is_empty() {
            return Err(ServiceError::ValidationError(
                "Phone cannot be empty".to_string(),
            ));
        }

        // Simple phone validation - could be more sophisticated
        if phone.len() < 10 || phone.len() > 20 {
            return Err(ServiceError::ValidationError(
                "Invalid phone number format".to_string(),
            ));
        }

        Ok(())
    }

    fn categorize_stores_by_rating(&self) -> HashMap<String, usize> {
        let mut categories = HashMap::new();

        for store in self.stores.values() {
            let category = match store.rating {
                r if r >= 4.5 => "excellent",
                r if r >= 4.0 => "very_good",
                r if r >= 3.5 => "good",
                r if r >= 3.0 => "average",
                _ => "below_average",
            };

            *categories.entry(category.to_string()).or_insert(0) += 1;
        }

        categories
    }

    fn init_sample_stores(&mut self) {
        let sample_stores = vec![
            Store::new(
                "FamilyMart Tokyo Station".to_string(),
                "Tokyo, Chiyoda, Marunouchi 1-9-1".to_string(),
                35.6812,
                139.7671,
                "24 hours".to_string(),
                "03-1234-5678".to_string(),
                vec!["convenience".to_string(), "24h".to_string()],
                '🏪',
            ),
            Store::new(
                "Matsumoto Kiyoshi Shinjuku".to_string(),
                "Tokyo, Shinjuku, Shinjuku 3-1-1".to_string(),
                35.6895,
                139.6917,
                "10:00-22:00".to_string(),
                "03-2345-6789".to_string(),
                vec!["pharmacy".to_string(), "cosmetics".to_string()],
                '🏪',
            ),
            Store::new(
                "Don Quijote Shibuya".to_string(),
                "Tokyo, Shibuya, Dogenzaka 2-25-5".to_string(),
                35.6580,
                139.6994,
                "24 hours".to_string(),
                "03-3456-7890".to_string(),
                vec!["variety".to_string(), "24h".to_string()],
                '🏪',
            ),
        ];

        for mut store in sample_stores {
            // Set some ratings
            store.rating = 4.0 + (store.id.len() % 10) as f64 * 0.1;
            self.stores.insert(store.id.clone(), store);
        }
    }
}

impl Default for StoreService {
    fn default() -> Self {
        Self::new()
    }
}

/// Store with distance information
#[derive(Debug, Clone)]
pub struct StoreDistance {
    pub store: Store,
    pub distance_km: f64,
}

/// Store statistics
#[derive(Debug, Clone)]
pub struct StoreStats {
    pub total_stores: usize,
    pub average_rating: f64,
    pub tag_counts: HashMap<String, usize>,
    pub stores_by_rating: HashMap<String, usize>,
}
