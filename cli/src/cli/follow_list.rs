use std::{collections::HashSet, hash::{Hash, Hasher}, fs, path::Path};

use pknames_wot::prediction::graph::WotGraph;
use serde::{Deserialize, Serialize, Serializer};

use super::wot_transformer::{WotTransformer, follow_lists_into_wot_graph};

#[derive(Serialize, Deserialize)]
pub struct FollowList {
    pub pubkey: String,
    #[serde(default = "default_alias")]
    pub alias: String,
    pub follows: Vec<Follow>,
}

fn default_alias() -> String{
    "".to_string()
}

impl std::fmt::Display for FollowList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut name = self.pubkey.clone();
        if self.alias.len() > 0 {
            name = format!("{} ({})", name, self.alias);
        }
        let follow_strings: Vec<String> = self
            .get_unique_follows()
            .iter()
            .map(|follow| format!("- {}", { follow }))
            .collect();
        write!(f, "List {}\n{}", name, follow_strings.join("\n"))
    }
}

impl FollowList {
    pub fn new(pubkey: &str, alias: &str) -> Self {
        FollowList {
            pubkey: pubkey.to_string(),
            alias: alias.to_string(),
            follows: vec![],
        }
    }
    pub fn new_with_follows(pubkey: String, alias: String , follows: Vec<Follow>) -> Self {
        FollowList {
            pubkey: pubkey,
            alias: alias,
            follows: follows,
        }
    }

    pub fn from_path(path: &Path) -> Result<Self, String> {
        let str_res = fs::read_to_string(path);
        if let Err(e) = str_res {
            return Err(format!("Failed to read list \"{}\". {}", path.to_str().unwrap(), e.to_string()));
        };
        let str = str_res.unwrap();
        let list = FollowList::from_json(&str);
        if let Err(e) = list {
            return Err(format!("Failed to parse list \"{}\". {}", path.to_str().unwrap(), e.to_string()));
        };
        Ok(list.unwrap())
    }

    pub fn get_unique_follows(&self) -> HashSet<&Follow> {
        let set: HashSet<&Follow> = HashSet::from_iter(self.follows.iter());
        set
    }

    /**
     * Retuns all pubkeys in this list
     */
    pub fn get_all_pubkeys(&self) -> HashSet<String> {
        let mut set: HashSet<String> = self.follows.iter().map(|follow| follow.pubkey().clone()).collect();
        set.insert(self.pubkey.clone());
        set
    }

    pub fn to_graph(lists: Vec<FollowList>) -> WotGraph {
        follow_lists_into_wot_graph(lists)
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
pub struct Follow(
    String, // pubkey
    #[serde(serialize_with = "serialize_weight")]
    f32, // weight
    #[serde(default = "default_domain")]
    #[serde(skip_serializing_if = "Option::is_none")]
    Option<String> // domain
);

fn serialize_weight<S>(weight: &f32, s: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let accuracy_after_comma = 3;
    let base: f32 = 10.0;
    let divider = base.powi(accuracy_after_comma);
    let clone = weight.clone();
    let rounded: f32 = (clone*divider).round()/divider;
    s.serialize_f32(rounded)
}

fn default_domain() -> Option<String> {
    None
}

impl std::fmt::Display for Follow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let emoji = match self.domain() {
            Some(_) => "🅰️ ",
            None => "📃"
        };
        let mut name = format!("{} {} {:.2}", emoji, self.pubkey(), self.1);

        if let Some(domain) = self.domain() {
            name = format!("{} {}", name, domain);
        };
        write!(f, "{}", name)
    }
}

impl Follow {
    pub fn new(target_pubkey: &str, weight: f32, domain: Option<String>) -> Self {
        Follow(target_pubkey.to_string(), weight, domain)
    }

    pub fn pubkey(&self) -> &String {
        &self.0
    }

    pub fn weight(&self) -> &f32 {
        &self.1
    }


    pub fn domain(&self) -> &Option<String> {
        &self.2
    }

}

impl PartialEq for Follow {
    fn eq(&self, other: &Self) -> bool {
        self.pubkey() == other.pubkey() && self.domain() == other.domain()
    }
}
impl Eq for Follow {}

impl Hash for Follow {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pubkey().hash(state);
        self.domain().hash(state);
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::follow_list::{FollowList, Follow};


    #[test]
    fn to_json_and_back() {
        let list = FollowList::new_with_follows(
            "pk:rcwgkobba4yupekhzxz6imtkyy1ph33emqt16fw6q6cnnbhdoqso".to_string(),
            "myList".to_string(),
            vec![
                Follow::new("pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy", 1.0/3.0, None),
                Follow::new("pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy", -1.0, Some("example.com".to_string()))
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
                0.33333334
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

    #[test]
    fn unique_follows() {
        let follows = vec![
            Follow("1".to_string(), 1.0, None),
            Follow("1".to_string(), 1.0, Some("example.com".to_string())),
            Follow("1".to_string(), 0.1, Some("example.com".to_string())),
        ];
        let list = FollowList{
            follows,
            pubkey: "me".to_string(),
            alias: "".to_string()
        };
        let unique: Vec<&Follow> = Vec::from_iter(list.get_unique_follows().into_iter());
        assert_eq!(unique.len(), 2);
    }
}
