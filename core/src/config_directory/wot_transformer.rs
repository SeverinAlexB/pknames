use std::collections::HashSet;
use crate::prediction::{node::{WotNode, WotFollow}, graph::WotGraph};
use super::follow_list::FollowList;


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
    use crate::{config_directory::{follow_list::FollowList, follow::Follow}, prediction::{graph::WotGraph, node::WotNode}};


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