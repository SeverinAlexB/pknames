use std::fmt::format;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FollowList {
    pubkey: String,
    alias: Option<String>,
    follows: Vec<Follow>,
}

impl std::fmt::Display for FollowList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut name = self.pubkey.clone();
        if let Some(alias) = self.alias.clone() {
            name = format!("{} {}", alias, name);
        }
        let follow_strings: Vec<String> = self
            .follows
            .iter()
            .map(|follow| format!("- {}", { follow }))
            .collect();
        write!(f, "List {}\n{}", name, follow_strings.join("\n"))
    }
}

impl FollowList {
    pub fn new(pubkey: String, alias: Option<String>) -> Self {
        FollowList {
            pubkey: pubkey,
            alias: alias,
            follows: vec![],
        }
    }
    pub fn new_with_follows(pubkey: String, alias: Option<String> , follows: Vec<Follow>) -> Self {
        FollowList {
            pubkey: pubkey,
            alias: alias,
            follows: follows,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let l: Result<FollowList, serde_json::Error> = serde_json::from_str(json);
        l
    }
}

#[derive(Serialize, Deserialize)]
pub struct Follow(String, f32, Option<String>);

impl std::fmt::Display for Follow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut name = self.0.clone();
        if let Some(alias) = self.2.clone() {
            name = format!("{} {}", alias, name);
        }
        write!(f, "{}: {}", name, self.1)
    }
}

impl Follow {
    pub fn new(target_pubkey: String, weight: f32, alias: Option<String>) -> Result<Self, &'static str> {
        if weight > 1.0 || weight < -1.0 {
            return Err("Weight not in range of -1.0 to 1.0.");
        }
        Ok(Follow(target_pubkey, weight, alias))
    }
}

#[cfg(test)]
mod tests {
    use crate::web_of_trust::follow_list::{Follow, FollowList};

    #[test]
    fn to_json_and_back() {
        let list = FollowList::new_with_follows(
            "pk:rcwgkobba4yupekhzxz6imtkyy1ph33emqt16fw6q6cnnbhdoqso".to_string(),
            Some("myList".to_string()),
            vec![
                Follow::new("pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy".to_string(), 1.0/3.0, None).unwrap(),
                Follow::new("pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy".to_string(), -1.0, Some("example.com".to_string())).unwrap()
            ],
        );

        let json = list.to_json();
        let recovered = FollowList::from_json(&json).unwrap();
        assert_eq!(list.pubkey, recovered.pubkey);
    }

    #[test]
    fn from_json() {
        let expected = r#"{
            "pubkey": "pk:rcwgkobba4yupekhzxz6imtkyy1ph33emqt16fw6q6cnnbhdoqso",
            "alias": "myList",
            "follows": [
              [
                "pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy",
                0.33333334,
                null
              ],
              [
                "pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy",
                -1.0,
                "example.com"
              ]
            ]
          }"#;
        let list = FollowList::from_json(expected).unwrap();
        assert_eq!(list.pubkey, "pk:rcwgkobba4yupekhzxz6imtkyy1ph33emqt16fw6q6cnnbhdoqso");
        assert_eq!(list.follows.len(), 2);
        assert_eq!(list.follows[0].0, "pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy");
    }
}
