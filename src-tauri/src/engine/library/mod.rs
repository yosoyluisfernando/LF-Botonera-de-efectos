pub mod browse;
pub mod browse_window;
pub mod catalog_count;
pub mod catalog_store;
pub mod catalog_tree;
mod embedded_tag_edit;
mod embedded_tags;
#[cfg(test)]
mod embedded_tags_tests;
mod file_name;
pub(crate) mod file_rename;
pub mod filesystem;
pub mod incremental;
mod index_discovery;
mod index_enrichment;
pub mod indexer;
pub mod metadata;
pub mod metadata_batch;
pub mod metadata_query;
pub mod metadata_store;
#[cfg(test)]
mod metadata_store_tests;
pub mod monitor;
mod path_store;
mod rename_journal;
#[cfg(test)]
mod retention_copy_tests;
mod root_merge;
mod root_path;
pub mod root_purge;
pub mod root_retention;
pub mod root_store;
pub mod scanner;
pub mod search;
mod search_expression;
mod search_index;
mod search_result;
mod search_score;
pub mod search_text;
pub mod service;
mod service_file;
mod service_metadata;
mod service_retention;
mod status;
mod tag_roles;
pub(crate) mod tag_write_journal;
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
#[cfg(test)]
mod metadata_real_copy_tests;
