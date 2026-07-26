pub mod browse;
pub mod catalog_store;
pub mod incremental;
mod index_discovery;
mod index_enrichment;
pub mod indexer;
pub mod metadata;
pub mod metadata_batch;
pub mod monitor;
mod root_merge;
mod root_path;
pub mod root_store;
pub mod scanner;
pub mod search;
mod search_expression;
mod search_score;
pub mod search_text;
pub mod service;
mod status;
mod tag_roles;
#[cfg(test)]
mod test_support;
mod time;

#[cfg(test)]
mod benchmark_tests;
#[cfg(test)]
mod category_benchmark_tests;
#[cfg(test)]
mod database_benchmark_tests;
#[cfg(test)]
mod metadata_benchmark_tests;
