use super::{Checkpointer, CheckpointerError};
use burn_core::{
    record::{FileRecorder, Record},
    tensor::backend::Backend,
};

/// The file checkpointer.
pub struct FileCheckpointer<FR> {
    directory: String,
    name: String,
    recorder: FR,
}

impl<FR> FileCheckpointer<FR> {
    /// Creates a new file checkpointer.
    ///
    /// # Arguments
    ///
    /// * `recorder` - The file recorder.
    /// * `directory` - The directory to save the checkpoints.
    /// * `name` - The name of the checkpoint.
    pub fn new(recorder: FR, directory: &str, name: &str) -> Self {
        std::fs::create_dir_all(directory).ok();

        Self {
            directory: directory.to_string(),
            name: name.to_string(),
            recorder,
        }
    }
    fn path_for_epoch(&self, epoch: usize) -> String {
        format!("{}/{}-{}", self.directory, self.name, epoch)
    }
}

impl<FR, R, B> Checkpointer<R, B> for FileCheckpointer<FR>
where
    R: Record<B>,
    FR: FileRecorder<B>,
    B: Backend,
{
    fn save(&self, epoch: usize, record: R) -> Result<(), CheckpointerError> {
        let file_path = self.path_for_epoch(epoch);
        log::info!("Saving checkpoint {} to {}", epoch, file_path);

        self.recorder
            .record(record, file_path.into())
            .map_err(CheckpointerError::RecorderError)?;

        Ok(())
    }

    fn restore(&self, epoch: usize, device: &B::Device) -> Result<R, CheckpointerError> {
        let file_path = self.path_for_epoch(epoch);
        log::info!("Restoring checkpoint {} from {}", epoch, file_path);
        let record = self
            .recorder
            .load(file_path.into(), device)
            .map_err(CheckpointerError::RecorderError)?;

        Ok(record)
    }

    fn delete(&self, epoch: usize) -> Result<(), CheckpointerError> {
        let file_to_remove = format!("{}.{}", self.path_for_epoch(epoch), FR::file_extension(),);

        if std::path::Path::new(&file_to_remove).exists() {
            log::info!("Removing checkpoint {}", file_to_remove);
            std::fs::remove_file(file_to_remove).map_err(CheckpointerError::IOError)?;
        }

        Ok(())
    }
}
