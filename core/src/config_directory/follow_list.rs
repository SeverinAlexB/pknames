use std::{collections::HashSet, fs, path::Path};
use serde::{Deserialize, Serialize};

use crate::prediction::{graph::WotGraph, node::{WotFollow, WotNode}};

use super::follow::Follow;


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

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let l: Result<FollowList, serde_json::Error> = serde_json::from_str(json);
        l
    }
}


impl Into<WotGraph> for Vec<FollowList> {
    fn into(self) -> WotGraph {
        let mut list_nodes: Vec<WotNode> = self.iter().map(|list| {
            let follows: Vec<WotFollow> = list.get_unique_follows().iter().map(|follow| {
                WotFollow::new(list.pubkey.clone(), follow.pubkey().clone(), follow.weight().clone(), follow.domain().clone())
            }).collect();
            let node = WotNode::new_list(list.pubkey.clone(), list.alias.clone(), follows);
            node
        }).collect();

        let existing_pubkeys: HashSet<String> = list_nodes.iter().map(|node| node.pubkey.clone()).collect();
        let all_pubkeys: HashSet<String> = self.iter().map(|list| list.get_all_pubkeys()).flatten().collect();

        let diff: Vec<String> = all_pubkeys.difference(&existing_pubkeys).map(|pubkey| pubkey.clone()).collect();
        let mut missing_nodes = diff.into_iter().map(|pubkey| WotNode::new_list(pubkey, "".to_string(), vec![])).collect::<Vec<WotNode>>();
        let mut result = vec![];
        result.append(&mut list_nodes);
        result.append(&mut missing_nodes);

        WotGraph::new(result)
    }
}



#[cfg(test)]
mod tests {
    use crate::{config_directory::{follow_list::{FollowList}, follow::Follow}, prediction::graph::WotGraph};


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

    #[test]
    fn transform_nodes() {
        let list1 = FollowList::new_with_follows(
            "pk:rcwgkobba4yupekhzxz6imtkyy1ph33emqt16fw6q6cnnbhdoqso".to_string(),
            "me".to_string(),
            vec![
                Follow::new("pk:1bdbmmxenbxuybfai88f1xg1djrpujxix5hw6fh9am7f4x5wapey", 1.0, None),
                Follow::new("pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy", 1.0/3.0, None),
                Follow::new("pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy", -1.0, Some("example.com1".to_string())),
                Follow::new("pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy", 1.0/3.0, None),
                Follow::new("pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy", -1.0, Some("example.com1".to_string())),
                Follow::new("pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy", -1.0, Some("example.com2".to_string()))
            ],
        );
        let list2 = FollowList::new_with_follows(
            "pk:1bdbmmxenbxuybfai88f1xg1djrpujxix5hw6fh9am7f4x5wapey".to_string(),
            "Alice".to_string(),
            vec![
                Follow::new("pk:s9y93dtpoibsfcnct35onkeyuiup9dfxwpftgerdqd7u84jcmkfy", 1.0/3.0, None),
                Follow::new("pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy", -1.0, Some("example.com1".to_string())),
                Follow::new("pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy", 1.0/3.0, None),
                Follow::new("pk:rcwgkobba4yupekhzxz6imtkyy1ph33emqt16fw6q6cnnbhdoqso", 1.0/3.0, None),
            ],
        );
        let graph: WotGraph = vec![list1, list2].into();
        assert_eq!(graph.nodes.len(), 5);
        let first = graph.get_node("pk:rcwgkobba4yupekhzxz6imtkyy1ph33emqt16fw6q6cnnbhdoqso").unwrap();
        let follows = first.follows.clone();
        assert_eq!(follows.len(), 6);
        // assert!(follows[0].attribution.is_none());
        // assert_eq!(follows.get(2).unwrap().clone().attribution.unwrap(), "example.com1".to_string());

        let _second = graph.get_node("pk:1bdbmmxenbxuybfai88f1xg1djrpujxix5hw6fh9am7f4x5wapey").unwrap();
    }

    #[test]
    fn pubkey_is_list_and_domain() {
        let list1 = FollowList::new_with_follows(
            "me".to_string(),
            "me".to_string(),
            vec![
                Follow::new("d2", 1.0, None),
                Follow::new("d2", 0.5, Some("example.com".to_string())),
            ],
        );
        let list2 = FollowList::new_with_follows(
            "d2".to_string(),
            "d2".to_string(),
            vec![
                Follow::new("d1", 0.5, Some("example.com".to_string())),
            ],
        );

        let graph: WotGraph = vec![list1, list2].into();

        assert_eq!(graph.nodes.len(), 3);
    }
}
