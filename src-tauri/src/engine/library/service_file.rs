use super::file_rename;
use super::service::LibraryService;
use crate::engine::persist::config_io;
use crate::model::AppConfig;

impl LibraryService {
    pub fn rename_file(
        &self,
        config: &mut AppConfig,
        path: &str,
        new_file_name: &str,
    ) -> Result<(String, String), String> {
        let _guard = self
            .operation
            .lock()
            .map_err(|_| "library_operation_lock")?;
        file_rename::rename_at(
            self.database_path(),
            &config_io::get_data_dir(),
            config,
            path,
            new_file_name,
        )
    }
}
