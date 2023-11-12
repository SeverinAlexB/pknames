
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
            WotNodeType::WotClass { name } => {
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
}




// Follow
#[derive(Debug, Clone)]
pub struct WotFollow {
    pub target_pubkey: String,
    pub source_pubkey: String,
    pub weight: f32
}
