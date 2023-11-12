
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum WotNodeType {
    WotFollowNode {
        follows: Vec<WotFollow>
    },
    WotClass {
        name: String
    },
    
    WotTempNode {
        follows: Vec<WotFollow>
    }
}

impl WotNodeType {
    pub fn get_follows(&self) -> Option<&Vec<WotFollow>> {
        match self {
            WotNodeType::WotClass { name: _ } => {
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

#[derive(Debug, Clone)]
pub struct WotNode {
    pub pubkey: String,
    pub typ: WotNodeType
}

impl WotNode {
    pub fn get_follow(&self, target_pubkey: &str) -> Option<&WotFollow> {
        let follows = self.typ.get_follows()?;
        let found = follows.iter().find(|&follow| follow.target_pubkey == target_pubkey);
        found
    }

    pub fn get_follows(&self) -> Option<&Vec<WotFollow>> {
        self.typ.get_follows()
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


#[cfg(test)]
mod tests {
    use super::{WotNode, WotNodeType};

    #[test]
    fn sort_node_vec() {
        let mut list = vec![
            WotNode {
                pubkey: "c".to_string(),
                typ: WotNodeType::WotFollowNode { follows: vec![] }
            },
            WotNode {
                pubkey: "b".to_string(),
                typ: WotNodeType::WotFollowNode { follows: vec![] }
            },
            WotNode {
                pubkey: "a".to_string(),
                typ: WotNodeType::WotFollowNode { follows: vec![] }
            },
            WotNode {
                pubkey: "d".to_string(),
                typ: WotNodeType::WotFollowNode { follows: vec![] }
            },
        ];

        list.sort_unstable_by_key(|node| node.pubkey.clone());

        assert_eq!(list[0].pubkey, "a");
        assert_eq!(list[1].pubkey, "b");
        assert_eq!(list[2].pubkey, "c");
        assert_eq!(list[3].pubkey, "d");
    }
}