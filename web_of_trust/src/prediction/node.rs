use std::fmt;
use std::hash::Hash;



#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum WotNodeType<F, C> {
    WotFollowNode {
        follows: Vec<WotFollow>,
        data: F
    },
    WotClass {
        data: C
    },
    
    WotTempNode {
        follows: Vec<WotFollow>
    }
}

impl<F, C> WotNodeType<F, C> where F: Clone, C: Clone {
    pub fn get_follows(&self) -> Option<&Vec<WotFollow>> {
        match self {
            WotNodeType::WotClass{..} => {
                None
            },
            WotNodeType::WotFollowNode {follows, ..} => {
                Some(follows)
            },
            WotNodeType::WotTempNode { follows } => {
                Some(follows)
            }
        }
    }

    pub fn get_follows_mut(&mut self) -> Option<&mut Vec<WotFollow>> {
        match self {
            WotNodeType::WotClass{..} => {
                None
            },
            WotNodeType::WotFollowNode {follows, ..} => {
                Some(follows)
            },
            WotNodeType::WotTempNode { follows } => {
                Some(follows)
            }
        }
    }

    /**
     * Remove all follows
     */
    pub fn clear_follows(&mut self) {
        match self {
            WotNodeType::WotClass{..} => {},
            WotNodeType::WotFollowNode {follows, ..} => {
                follows.clear();
            },
            WotNodeType::WotTempNode { follows } => {
                follows.clear();
            }
        };
    }

    /**
     * Extend follows
     */
    pub fn extend_follows(&mut self, new_follows: Vec<WotFollow>) {
        if new_follows.len() == 0 {
            return;
        }
        match self {
            WotNodeType::WotClass{..} => {
                panic!("Can't set follows of a WotClass node.")
            },
            WotNodeType::WotFollowNode {follows, ..} => {
                follows.extend(new_follows.into_iter());
            },
            WotNodeType::WotTempNode { follows } => {
                follows.extend(new_follows.into_iter());
            }
        };
    }

}

impl<F, C> fmt::Display for WotNodeType<F, C> where F: Clone, C: Clone {
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
            WotNodeType::WotTempNode { .. } => {
                write!(f, "temp follows {:?}", follow_str)
            },
            WotNodeType::WotClass { .. } => {
                write!(f, "class")
            },
            WotNodeType::WotFollowNode { .. } => {
                write!(f, "follows {}", follow_str)
            }
        }
        
    }
}

#[derive(Debug, Clone)]
pub struct WotNode<F, C>
where F: Clone, C: Clone{
    pub pubkey: String,
    pub alias: String,
    pub typ: WotNodeType<F, C>
}

impl<F, C> fmt::Display for WotNode<F, C> where F: Clone, C: Clone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut name = self.pubkey.clone();
        if self.alias.len() > 0 {
            name = format!("{} ({})", self.alias, name);
        }
        write!(f, "{} {}", name, self.typ)
    }
}

impl<F, C> WotNode<F, C> where F: Clone, C: Clone {

    pub fn new_class(pubkey: String, alias: String, data: C) -> WotNode<F, C> {
        WotNode {
            pubkey,
            alias,
            typ: WotNodeType::WotClass { data: data }
        }
    }

    pub fn new_list(pubkey: String, alias: String, follows: Vec<WotFollow>, data: F) -> WotNode<F, C> {
        WotNode {
            pubkey,
            alias,
            typ: WotNodeType::WotFollowNode { follows, data }
        }
    }

    pub fn get_follow(&self, target_pubkey: &str) -> Option<&WotFollow> {
        let follows = self.typ.get_follows()?;
        let found = follows.iter().find(|&follow| follow.target_pubkey == target_pubkey);
        found
    }

    pub fn get_follow_mut(&mut self, target_pubkey: &str) -> Option<&mut WotFollow> {
        let follows = self.typ.get_follows_mut()?;
        let found = follows.iter_mut().find(| follow| follow.target_pubkey == target_pubkey);
        found
    }

    pub fn get_follows(&self) -> Option<&Vec<WotFollow>> {
        self.typ.get_follows()
    }

    /**
     * Finds a WotNode in a Vec<&WotNode>
     */
    pub fn binary_search_ref<'a>(pubkey: &str, list: &'a Vec<&WotNode<F, C>>) -> Option<&'a WotNode<F, C>> {
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
    pub fn binary_search<'a>(pubkey: &str, list: &'a Vec<WotNode<F, C>>) -> Option<&'a WotNode<F, C>> {
        let result = list.binary_search_by_key(&pubkey, |node| &node.pubkey);
        if let Ok(index) = result  {
            let node = &list[index];
            Some(node)
        } else {
            None
        }
    }



}

impl<A, B> PartialEq for WotNode<A, B> where A: Clone, B: Clone {
    fn eq(&self, other: &Self) -> bool {
        self.pubkey == other.pubkey
    }
}

impl<A, B> Eq for WotNode<A, B> where A: Clone, B:Clone {}

impl<A, B> Hash for WotNode<A, B> where A: Clone, B: Clone {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pubkey.hash(state);
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

impl fmt::Display for WotFollow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {} {}", self.source_pubkey, self.target_pubkey, self.weight)
    }
}

impl PartialEq for WotFollow {
    fn eq(&self, other: &Self) -> bool {
        self.target_pubkey == other.target_pubkey && self.source_pubkey == other.source_pubkey
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

    use super::{WotNode, WotNodeType};

    #[test]
    fn sort_node_vec() {
        let mut list: Vec<WotNode<(), ()>> = vec![
            WotNode::new_list(
                "c".to_string(), 
                "".to_string(), 
                vec![], 
                ()
            ),
            WotNode::new_list(
                "b".to_string(), 
                "".to_string(), 
                vec![], 
                ()
            ),
            WotNode::new_list(
                "a".to_string(), 
                "".to_string(), 
                vec![], 
                ()
            ),
            WotNode::new_list(
                "d".to_string(), 
                "".to_string(), 
                vec![], 
                ()
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
        let node: WotNode<(), ()> = WotNode::new_list(
            pubkey.clone(), 
            "me".to_string(), 
            vec![
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
            ], 
            ()
        );
        println!("{}", node);

    }
}