use std::time::Duration;

use pkarr::{dns::Packet, PublicKey};
use ttl_cache::TtlCache;




/**
 * Pkarr record ttl cache
 */
pub struct PkarrPacketTtlCache{
    cache: TtlCache<String, Vec<u8>>
}

impl PkarrPacketTtlCache {
    pub fn new() -> Self {
        PkarrPacketTtlCache{
            cache: TtlCache::new(100)
        }
    }

    /**
     * Adds packet and caches it for the ttl the least long lived answer is valid for.
     */
    pub fn add(&mut self, pubkey: PublicKey, reply: Vec<u8>) {
        let default_ttl = 1200;
        let packet = Packet::parse(&reply).unwrap();
        let min_ttl = packet.answers.iter().map(|answer| answer.ttl).min().unwrap_or(default_ttl);

        let ttl = 60.max(min_ttl); // At least 1min
        let ttl = Duration::from_secs(ttl as u64);

        self.cache.insert(pubkey.to_z32(), reply, ttl);
    }

    pub fn get(&self, pubkey: &PublicKey) -> Option<Vec<u8>> {
        let z32 = pubkey.to_z32();
        self.cache.get(&z32).map(|value| value.clone())
    }

}