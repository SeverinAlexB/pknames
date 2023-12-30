use std::fs;
use std::path::{PathBuf};

use crate::cli::follow_list::FollowList;


pub struct StaticListsDirectory {
    pub path: PathBuf,
}

impl StaticListsDirectory {
    /**
     * Creates new ConfigFolder.
     */
    pub fn new(path: PathBuf) -> Self {
        StaticListsDirectory { path }
    }

    /**
     * Creates the directory if it does not exist.
     */
    pub fn create_if_it_does_not_exist(&self) -> Result<(), std::io::Error> {
        if self.path.exists() {
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
        fs::create_dir(&self.path)
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
        let paths = fs::read_dir(&self.path)?;

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
    use std::path::PathBuf;
    use crate::cli::follow_list::{FollowList, Follow};
    use super::StaticListsDirectory;


    #[test]
    fn read_lists() {
        let config = StaticListsDirectory::new(PathBuf::from("/tmp/fancydns827209438/static_lists"));
        let _ = config.delete(); // Delete so the test can work again even though it failed before.
        let _ = config.create_if_it_does_not_exist();

        let list = FollowList::new_with_follows(
            "pk:rcwgkobba4yupekhzxz6imtkyy1ph33emqt16fw6q6cnnbhdoqso".to_string(),
            "myList".to_string(),
            vec![
                Follow::new(
                    "pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy".to_string(),
                    1.0 / 3.0,
                    None
                )
                .unwrap(),
                Follow::new(
                    "pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy".to_string(),
                    -1.0,
                    Some("example.com".to_string()),
                )
                .unwrap(),
            ],
        );

        let json = list.to_json();
        let mut path = config.path.clone();
        path.push("myList.json");
        std::fs::write(path, json).unwrap();

        let lists = config.read_lists().unwrap();
        assert_eq!(lists.len(), 1)
    }
}
