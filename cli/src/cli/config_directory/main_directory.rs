use std::fs;
use std::path::{Path, PathBuf};
use zbase32;
use pkarr::Keypair;

use super::static_lists_directory::StaticListsDirectory;

const SECRET_KEY_LENGTH: usize = 32;

pub struct MainDirectory {
    pub path: PathBuf,
    pub static_lists_dir: StaticListsDirectory
}

impl MainDirectory {
    /**
     * Creates new ConfigFolder.
     */
    pub fn new(path: PathBuf) -> Self {
        let mut static_lists_path = path.clone().into_os_string();
        static_lists_path.push("/static_lists");
        let static_lists_dir = StaticListsDirectory::new(static_lists_path.into());
        MainDirectory { 
            path,
            static_lists_dir
        }
    }

    /**
     * Creates new ConfigFolder.
     */
    pub fn new_by_string(path: &str) -> Self {
        let expanded = shellexpand::tilde(path);
        let full_path: String = expanded.into();

        let path = Path::new(&full_path);
        let path_buf = PathBuf::from(path);
        MainDirectory::new(path_buf)
    }

    /**
     * Creates the directory if it does not exist. At least given parent folder of `path` must exist otherwise it will throw an error.
     */
    fn create_main_dir_if_it_does_not_exist(&self) -> Result<(), std::io::Error> {
        if self.path.exists() {
            if self.path.is_dir() {
                return Ok(());
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Not found",
                ));
            }
        };

        // Folder does not exist. Let's check if we can create the folder in the parent directory.
        let parent = self.path.parent();
        if parent.is_none() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Not found",
            ));
        };
        let parent_buf = PathBuf::from(parent.unwrap());
        if !parent_buf.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Not found",
            ));
        };
        // Create main
        fs::create_dir(&self.path)
    }

    /**
     * Creates the directory if it does not exist. At least given parent folder of `path` must exist otherwise it will throw an error.
     */
    pub fn create_if_it_does_not_exist(&self) -> Result<(), std::io::Error> {
        self.create_main_dir_if_it_does_not_exist()?;
        self.static_lists_dir.create_if_it_does_not_exist()
    }

    /**
     * Deletes the whole directory tree. Careful!
     */
    pub fn delete(&self) -> Result<(), std::io::Error> {
        fs::remove_dir_all(&self.path)
    }

    pub fn get_keypair_path(&self) -> PathBuf {
        let mut path = self.path.clone().into_os_string();
        path.push("/secret");
        PathBuf::from(path)
    }

    /**
     * Reads the keypair from the disk
     */
    pub fn read_keypair(&self) -> Result<Keypair, String> {
        let path = self.get_keypair_path();

        let read_result = fs::read_to_string(path);
        if let Err(e) = read_result {
            return Err(e.to_string());
        };
        
        let file_content = read_result.unwrap();
        let decode_result = zbase32::decode_full_bytes_str(&file_content);
        if let Err(e) = decode_result {
            return Err(e.to_string());
        };

        let plain_secret = decode_result.unwrap();
        if plain_secret.len() != SECRET_KEY_LENGTH {
            return Err(format!("Secret not {} bytes long.", SECRET_KEY_LENGTH));
        }
        let slice: &[u8; SECRET_KEY_LENGTH] = &plain_secret[0..SECRET_KEY_LENGTH].try_into().unwrap();
        let keypair = Keypair::from_secret_key(slice);
        Ok(keypair)
    }

    /**
     * Creates a random keypair and writes it to the disk
     */
    pub fn create_random_keypair(&self) -> Result<Keypair, String> {
        let keypair = Keypair::random();
        let encoded = zbase32::encode_full_bytes(&keypair.secret_key());

        let path = self.get_keypair_path();
        let result = fs::write(path, encoded);
        
        match result {
            Ok(_) => Ok(keypair),
            Err(e) => Err(e.to_string())
        }
    }

    /**
     * Reads the keypair from the disk or if it does not exist, creates one.
     */
    pub fn read_or_create_keypair(&self) -> Keypair {
        if self.get_keypair_path().exists() {
            self.read_keypair().unwrap()
        } else {
            self.create_random_keypair().unwrap()
        }
    }

}

#[cfg(test)]
mod tests {
    use pkarr::Keypair;
    use zbase32;
    use super::MainDirectory;

    #[test]
    fn create_if_it_does_not_exist() {
        let config = MainDirectory::new_by_string("/tmp/fancydns827209438");
        let _ = config.delete(); // Delete so the test can work again even though it failed before.
        assert_eq!(config.path.exists(), false);
        let result = config.create_if_it_does_not_exist();
        assert!(result.is_ok());
        assert_eq!(config.path.exists(), true);
        assert_eq!(config.static_lists_dir.path.exists(), true);
        config.delete().unwrap();
        assert_eq!(config.path.exists(), false);
    }

    #[test]
    fn create_and_read_keypair() {
        let main = MainDirectory::new_by_string("/tmp/fancydns827209438");
        let _ = main.delete(); // Delete so the test can work again even though it failed before.
        main.create_if_it_does_not_exist().unwrap();
        let keypair = main.create_random_keypair().unwrap();
        let read_keypair = main.read_keypair().unwrap();
        assert_eq!(keypair.secret_key(), read_keypair.secret_key());
    }

    #[test]
    fn read_or_create_keypair() {
        let main = MainDirectory::new_by_string("/tmp/fancydns827209438");
        let keypair = main.read_or_create_keypair();
        println!("{}", keypair.to_z32())
    }
}
