use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize, Serializer};


#[derive(Serialize, Deserialize)]
pub struct Follow(
    pub String, // pubkey
    #[serde(serialize_with = "serialize_weight")]
    pub f32, // weight
    #[serde(default = "default_domain")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub Option<String> // domain
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
