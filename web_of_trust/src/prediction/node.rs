use std::fmt;
use std::hash::Hash;



#[derive(Debug, Clone)]
pub struct WotNode {
    pub pubkey: String,
    pub alias: String,
    pub follows: Vec<WotFollow>
}

impl fmt::Display for WotNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut name = self.pubkey.clone();
        if self.alias.len() > 0 {
            name = format!("{} ({})", name, self.alias);
        }
        write!(f, "{}", name)
    }
}


impl WotNode {

    pub fn new_class(pubkey: String, alias: String) -> WotNode {
        WotNode {
            pubkey,
            alias,
            follows: vec![]
        }
    }

    pub fn new_list(pubkey: String, alias: String, follows: Vec<WotFollow>) -> WotNode {
        WotNode {
            pubkey,
            alias,
            follows: follows 
        }
    }

    /**
     * Extend follows
     */
    pub fn extend_follows(&mut self, new_follows: Vec<WotFollow>) {
        if new_follows.len() == 0 {
            return;
        }
        self.follows.extend(new_follows.into_iter());
    }

    pub fn get_follow(&self, target_pubkey: &str) -> Option<&WotFollow> {
        let found = self.follows.iter().find(|&follow| follow.target_pubkey == target_pubkey);
        found
    }

    pub fn get_follow_mut(&mut self, target_pubkey: &str) -> Option<&mut WotFollow> {
        let found = self.follows.iter_mut().find(| follow| follow.target_pubkey == target_pubkey);
        found
    }

    /**
     * Finds a WotNode in a Vec<&WotNode>
     */
    pub fn binary_search_ref<'a>(pubkey: &str, list: &'a Vec<&WotNode>) -> Option<&'a WotNode> {
        let result = list.binary_search_by_key(&pubkey, |node| &node.pubkey);
        if let Ok(index) = result  {
            let node = &list[index];
            Some(node)
        } else {
            None
        }
    }

    /**
     * Finds a WotNode in a Vec<WotNode>
     */
    pub fn binary_search<'a>(pubkey: &str, list: &'a Vec<WotNode>) -> Option<&'a WotNode> {
        let result = list.binary_search_by_key(&pubkey, |node| &node.pubkey);
        if let Ok(index) = result  {
            let node = &list[index];
            Some(node)
        } else {
            None
        }
    }



}

impl PartialEq for WotNode {
    fn eq(&self, other: &Self) -> bool {
        self.pubkey == other.pubkey
    }
}

impl Eq for WotNode {}

impl Hash for WotNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pubkey.hash(state);
    }
}

// Follow
#[derive(Debug, Clone)]
pub struct WotFollow {
    pub target_pubkey: String,
    pub source_pubkey: String,
    pub weight: f32,
    pub attribution: Option<String>
}

impl WotFollow {
    pub fn new(source_pubkey: String, target_pubkey: String, weight: f32, attribution: Option<String>) -> Self {
        WotFollow { target_pubkey, source_pubkey, weight , attribution}
    }
}

impl fmt::Display for WotFollow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {} {}", self.source_pubkey, self.target_pubkey, self.weight)
    }
}

impl PartialEq for WotFollow {
    fn eq(&self, other: &Self) -> bool {
        self.target_pubkey == other.target_pubkey && self.source_pubkey == other.source_pubkey && self.attribution == other.attribution
    }
}

impl Eq for WotFollow {}

impl Hash for WotFollow {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.source_pubkey.hash(state);
        self.target_pubkey.hash(state);
    }
}


#[cfg(test)]
mod tests {
    use crate::prediction::node::WotFollow;
    use super::WotNode;

    #[test]
    fn sort_node_vec() {
        let mut list: Vec<WotNode> = vec![
            WotNode::new_list(
                "c".to_string(), 
                "".to_string(), 
                vec![]
            ),
            WotNode::new_list(
                "b".to_string(), 
                "".to_string(), 
                vec![]
            ),
            WotNode::new_list(
                "a".to_string(), 
                "".to_string(), 
                vec![]
            ),
            WotNode::new_list(
                "d".to_string(), 
                "".to_string(), 
                vec![]
            ),
        ];

        list.sort_unstable_by_key(|node| node.pubkey.clone());

        assert_eq!(list[0].pubkey, "a");
        assert_eq!(list[1].pubkey, "b");
        assert_eq!(list[2].pubkey, "c");
        assert_eq!(list[3].pubkey, "d");
    }


    #[test]
    fn display_node() {
        let pubkey = String::from("923jladsf");
        let node: WotNode = WotNode::new_list(
            pubkey.clone(), 
            "me".to_string(), 
            vec![
                WotFollow {
                    source_pubkey: String::from("hello"),
                    target_pubkey: String::from("n1"),
                    weight: 1.0,
                    attribution: Some("example.com".to_string())
                },
                WotFollow {
                    source_pubkey: String::from("hello"),
                    target_pubkey: String::from("n2"),
                    weight: -1.0,
                    attribution: Some("example.com".to_string())
                }
            ]
        );
        println!("{}", node);

    }
}