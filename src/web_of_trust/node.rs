

pub trait WotNodeTrait {
    fn get_pubkey(&self) -> &str;
}

impl std::fmt::Debug for dyn WotNodeTrait {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Node {}", self.get_pubkey())
    }
}

// Follow
#[derive(Debug)]
pub struct WotFollow {
    pub target_pubkey: String,
    pub source_pubkey: String,
    pub weight: f32
}


// Node
#[derive(Debug)]
pub struct WotNode {
    pub pubkey: String,
    pub follows: Vec<WotFollow>
}

impl WotNode {
    pub fn get_follow(&self, target_pubkey: &str) -> Option<&WotFollow> {
        let found = self.follows.iter().find(|&follow| follow.target_pubkey == target_pubkey);
        found
    }
}

impl WotNodeTrait for WotNode {
    fn get_pubkey(&self) -> &str {
        &self.pubkey
    }
}

// Temp node
#[derive(Debug)]
pub struct WotTempNode(WotNode);

// Class
#[derive(Debug)]
pub struct WotClass {
    pub pubkey: String,
    pub name: Option<String>,
}

impl WotNodeTrait for WotClass {
    fn get_pubkey(&self) -> &str {
        &self.pubkey
    }
}
