use std::fs;
use std::path::{Path, PathBuf};

use super::follow_list::FollowList;


pub struct ConfigFolder {
    pub path: PathBuf,
}

impl ConfigFolder {
    /**
     * Creates new ConfigFolder.
     */
    pub fn new(path: PathBuf) -> Self {
        ConfigFolder { path }
    }

    /**
     * Creates new ConfigFolder.
     */
    pub fn new_by_string(path: &str) -> Self {
        let expanded = shellexpand::tilde(path);
        let full_path: String = expanded.into();

        let path = Path::new(&full_path);
        let path_buf = PathBuf::from(path);
        ConfigFolder::new(path_buf)
    }

    /**
     * Path to the list folder.
     */
    pub fn get_lists_path(&self) -> PathBuf {
        let mut main_path = self.path.clone().into_os_string();
        main_path.push("/lists");
        let lists_path: PathBuf = main_path.into();
        lists_path
    }

    fn create_lists_dir_if_it_does_not_exist(&self) -> Result<(), std::io::Error> {
        let path = self.get_lists_path();
        if path.exists() {
            if self.path.is_dir() {
                return Ok(());
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Expected directory.",
                ));
            }
        };
        // Create dir
        fs::create_dir(&path)
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
        fs::create_dir(&self.path)?;

        // Create lists dir
        let list_path = self.get_lists_path();
        fs::create_dir(&list_path)
    }

    /**
     * Creates the directory if it does not exist. At least given parent folder of `path` must exist otherwise it will throw an error.
     */
    pub fn create_if_it_does_not_exist(&self) -> Result<(), std::io::Error> {
        self.create_main_dir_if_it_does_not_exist()?;
        self.create_lists_dir_if_it_does_not_exist()
    }

    /**
     * Deletes the whole directory tree. Careful!
     */
    pub fn delete(&self) -> Result<(), std::io::Error> {
        fs::remove_dir_all(&self.path)
    }

    /**
     * Returns the lists that can be read sucessfully.
     */
    pub fn read_valid_lists(&self) -> Vec<FollowList> {
        let result = self.read_lists();
        if result.is_err() {
            return vec![]
        };
        let result_list = result.unwrap();

        let valid_lists: Vec<FollowList> = result_list.into_iter().filter_map(|res| {
            if res.is_ok() {
                Some(res.unwrap())
            } else {
                None
            }
        }).collect();

        valid_lists
    }

    pub fn read_lists(&self) -> Result<Vec<Result<FollowList, String>>, std::io::Error> {
        let paths = fs::read_dir(self.get_lists_path())?;

        let paths: Vec<PathBuf> = paths
            .map(|entry| {
                let path = entry.unwrap().path();
                path
            })
            .filter(|path| {
                let extension_opt = path.extension();
                if let None = extension_opt {
                    return false;
                };
                let extension_opt = extension_opt.unwrap().to_str();
                if let None = extension_opt {
                    return false;
                };
                let extension = extension_opt.unwrap();

                let matching = extension == "json";
                matching
            })
            .collect();

        let lists: Vec<Result<FollowList, String>> = paths
            .into_iter()
            .map(|path| {
                let str_res = fs::read_to_string(&path);
                if let Err(e) = str_res {
                    return Err(format!("Failed to read list \"{}\". {}", path.to_str().unwrap(), e.to_string()));
                };
                let str = str_res.unwrap();
                let list = FollowList::from_json(&str);
                if let Err(e) = list {
                    return Err(format!("Failed to parse list \"{}\". {}", path.to_str().unwrap(), e.to_string()));
                };
                Ok(list.unwrap())
            })
            .collect();

        Ok(lists)
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::follow_list::{FollowList, Follow};

    use super::ConfigFolder;

    #[test]
    fn create_if_it_does_not_exist() {
        let config = ConfigFolder::new_by_string("/tmp/fancydns827209438");
        let _ = config.delete(); // Delete so the test can work again even though it failed before.
        assert_eq!(config.path.exists(), false);
        let result = config.create_if_it_does_not_exist();
        assert!(result.is_ok());
        assert_eq!(config.path.exists(), true);
        config.delete().unwrap();
        assert_eq!(config.path.exists(), false);
    }

    #[test]
    fn read_lists() {
        let config = ConfigFolder::new_by_string("/tmp/fancydns827209438");
        let _ = config.delete(); // Delete so the test can work again even though it failed before.
        let _ = config.create_if_it_does_not_exist();

        let list = FollowList::new_with_follows(
            "pk:rcwgkobba4yupekhzxz6imtkyy1ph33emqt16fw6q6cnnbhdoqso".to_string(),
            Some("myList".to_string()),
            vec![
                Follow::new(
                    "pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy".to_string(),
                    1.0 / 3.0,
                    "".to_string(),
                    None
                )
                .unwrap(),
                Follow::new(
                    "pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy".to_string(),
                    -1.0,
                    "".to_string(),
                    Some("example.com".to_string()),
                )
                .unwrap(),
            ],
        );

        let json = list.to_json();
        let mut path = config.get_lists_path();
        path.push("myList.json");
        std::fs::write(path, json).unwrap();

        let lists = config.read_lists().unwrap();
        assert_eq!(lists.len(), 1)
    }
}
