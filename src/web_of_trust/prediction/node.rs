use std::fmt;




#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum WotNodeType {
    WotFollowNode {
        follows: Vec<WotFollow>
    },
    WotClass,
    
    WotTempNode {
        follows: Vec<WotFollow>
    }
}

impl WotNodeType {
    pub fn get_follows(&self) -> Option<&Vec<WotFollow>> {
        match self {
            WotNodeType::WotClass => {
                None
            },
            WotNodeType::WotFollowNode {follows} => {
                Some(follows)
            },
            WotNodeType::WotTempNode { follows } => {
                Some(follows)
            }
        }
    }

    pub fn get_follows_mut(&mut self) -> Option<&mut Vec<WotFollow>> {
        match self {
            WotNodeType::WotClass => {
                None
            },
            WotNodeType::WotFollowNode {follows} => {
                Some(follows)
            },
            WotNodeType::WotTempNode { follows } => {
                Some(follows)
            }
        }
    }
}

impl fmt::Display for WotNodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let follows = self.get_follows();
        let follow_str = match follows {
            None => String::from(""),
            Some(follows) => {
                let strings: Vec<String> = follows.iter().map(|follow| format!("{}: {}", follow.target_pubkey, follow.weight)).collect();
                strings.join(", ")
            }
        };

        match self {
            WotNodeType::WotTempNode { follows } => {
                write!(f, "temp follows {:?}", follow_str)
            },
            WotNodeType::WotClass => {
                write!(f, "class")
            },
            WotNodeType::WotFollowNode { follows } => {
                write!(f, "follows {}", follow_str)
            }
        }
        
    }
}

#[derive(Debug, Clone)]
pub struct WotNode {
    pub pubkey: String,
    pub alias: Option<String>,
    pub typ: WotNodeType
}

impl fmt::Display for WotNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut name = self.pubkey.clone();
        if let Some(alias) = self.alias.clone() {
            name = format!("{} ({})",alias, name);
        }
        write!(f, "{} {}", name, self.typ)
    }
}

impl WotNode {
    pub fn get_follow(&self, target_pubkey: &str) -> Option<&WotFollow> {
        let follows = self.typ.get_follows()?;
        let found = follows.iter().find(|&follow| follow.target_pubkey == target_pubkey);
        found
    }

    pub fn get_follow_mut(&mut self, target_pubkey: &str) -> Option<&mut WotFollow> {
        let mut follows = self.typ.get_follows_mut()?;
        let found = follows.iter_mut().find(| follow| follow.target_pubkey == target_pubkey);
        found
    }

    pub fn get_follows(&self) -> Option<&Vec<WotFollow>> {
        self.typ.get_follows()
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

// Follow
#[derive(Debug, Clone)]
pub struct WotFollow {
    pub target_pubkey: String,
    pub source_pubkey: String,
    pub weight: f32
}

impl WotFollow {
    pub fn new(source_pubkey: String, target_pubkey: String, weight: f32) -> Result<Self, &'static str> {
        if weight < -1.0 || weight > 1.0 {
            return Err("Weight not in range of -1.0 to 1.0.")
        }
        Ok(WotFollow { target_pubkey, source_pubkey, weight })
    }
}


#[cfg(test)]
mod tests {
    use crate::web_of_trust::prediction::node::WotFollow;

    use super::{WotNode, WotNodeType};

    #[test]
    fn sort_node_vec() {
        let mut list = vec![
            WotNode {
                pubkey: "c".to_string(),
                alias: None,
                typ: WotNodeType::WotFollowNode { follows: vec![] }
            },
            WotNode {
                pubkey: "b".to_string(),
                alias: None,
                typ: WotNodeType::WotFollowNode { follows: vec![] }
            },
            WotNode {
                pubkey: "a".to_string(),
                alias: None,
                typ: WotNodeType::WotFollowNode { follows: vec![] }
            },
            WotNode {
                pubkey: "d".to_string(),
                alias: None,
                typ: WotNodeType::WotFollowNode { follows: vec![] }
            },
        ];

        list.sort_unstable_by_key(|node| node.pubkey.clone());

        assert_eq!(list[0].pubkey, "a");
        assert_eq!(list[1].pubkey, "b");
        assert_eq!(list[2].pubkey, "c");
        assert_eq!(list[3].pubkey, "d");
    }

    #[test]
    fn my_test() {
        let pubkey = String::from("hello");
        let node = WotNode {
            pubkey: pubkey.clone(),
            alias: None,
            typ: WotNodeType::WotFollowNode { follows: vec![] }
        };
        println!("{}", pubkey);

    }

    #[test]
    fn display_node() {
        let pubkey = String::from("923jladsf");
        let node = WotNode {
            pubkey: pubkey.clone(),
            alias: Some("me".to_string()),
            typ: WotNodeType::WotFollowNode { follows: vec![
                WotFollow {
                    source_pubkey: String::from("hello"),
                    target_pubkey: String::from("n1"),
                    weight: 1.0
                },
                WotFollow {
                    source_pubkey: String::from("hello"),
                    target_pubkey: String::from("n2"),
                    weight: -1.0
                }
            ] }
        };
        println!("{}", node);

    }
}