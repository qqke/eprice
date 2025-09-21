pub mod engine;
pub mod filters;
pub mod ui;

pub use engine::{SearchEngine, SearchQuery, SearchResult, SearchResultItem};
pub use filters::{CategoryFilter, PriceRange, SearchFilters, StoreFilter};
pub use ui::AdvancedSearchUI;
