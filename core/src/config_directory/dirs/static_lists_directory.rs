use std::fs;
use std::path::PathBuf;

use crate::config_directory::follow_list::FollowList;


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
    pub fn create_if_it_does_not_exist(&self, me_pubkey: &str) -> Result<(), std::io::Error> {
        if self.path.exists() && self.path.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Expected directory.",
            ));
        } else if !self.path.exists() {
            fs::create_dir(&self.path)?;
        };


        // Create default me list
        let me_list_path = self.derive_filename(me_pubkey);
        if !me_list_path.exists() {
            let new_list = FollowList::new(me_pubkey, "me");
            self.write_list(me_pubkey, new_list)?;
        };

        Ok(())
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
                FollowList::from_path(&path)
            })
            .collect();

        Ok(lists)
    }

    fn derive_filename(&self, pubkey: &str) -> PathBuf {
        let pubkey_without_pk = pubkey.replace("pk:", "");
        let path = self.path.clone().join(format!("{}.json", pubkey_without_pk));
        path
    }

    /**
     * Read list from disk.
     */
    pub fn read_list(&self, pubkey: &str) -> Result<FollowList, String> {
        let path = self.derive_filename(pubkey);
        FollowList::from_path(&path)
    }

    /**
     * Write list to disk
     */
    pub fn write_list(&self, pubkey: &str, list: FollowList) -> Result<(),std::io::Error> {
        let path = self.derive_filename(pubkey);
        let str = list.to_json();
        fs::write(path, str)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::config_directory::{follow_list::FollowList, follow::Follow};
    use super::StaticListsDirectory;


    #[test]
    fn read_lists() {
        let config = StaticListsDirectory::new(PathBuf::from("/tmp/fancydns827209438"));
        let _ = config.delete(); // Delete so the test can work again even though it failed before.
        let _ = config.create_if_it_does_not_exist("test");

        let list = FollowList::new_with_follows(
            "pk:rcwgkobba4yupekhzxz6imtkyy1ph33emqt16fw6q6cnnbhdoqso",
            "myList",
            vec![
                Follow::new(
                    "pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy",
                    1.0 / 3.0,
                    None
                ),
                Follow::new(
                    "pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy",
                    -1.0,
                    Some("example.com"),
                ),
            ],
        );

        let json = list.to_json();
        let mut path = config.path.clone();
        path.push("myList.json");
        std::fs::write(path, json).unwrap();

        let lists = config.read_lists().unwrap();
        assert_eq!(lists.len(), 1)
    }

    #[test]
    fn read_empty_me_list() {
        let config = StaticListsDirectory::new(PathBuf::from("/tmp/fancydns827209438"));
        let _ = config.delete(); // Delete so the test can work again even though it failed before.
        let res = config.create_if_it_does_not_exist("test").unwrap();

        let list_result = config.read_list("test");

        assert!( list_result.is_ok());
        let list = list_result.unwrap();
        assert_eq!(list.pubkey, "test");
        assert_eq!(list.alias, "me");
        assert_eq!(list.follows.len(), 0);
    }
}
