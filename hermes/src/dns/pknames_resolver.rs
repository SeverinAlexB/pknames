use super::{protocol::{QueryType, DnsPacket}, resolve::ResolveError};

pub fn resolve_pknames(qname: &str, qtype: QueryType) -> Result<DnsPacket, ResolveError> {
    Err(ResolveError::NoServerFound)
}