use std::collections::HashSet;

use fancyd_wot::prediction::{node::{WotNode, WotFollow}, graph::WotGraph};

use super::follow_list::FollowList;

pub struct WotTransformer {
    lists: Vec<FollowList>
}

impl WotTransformer {
    pub fn new(lists: Vec<FollowList>) -> Self {
        WotTransformer{
            lists
        }
    }

    /**
     * Extracts nodes from lists
     */
    pub fn get_follow_nodes(&self) -> Vec<WotNode> {
        let mut list_nodes: Vec<WotNode> = self.lists.iter().map(|list| {
            let follows: Vec<WotFollow> = list.get_unique_follows().iter().map(|follow| {
                WotFollow::new(list.pubkey.clone(), follow.pubkey().clone(), follow.weight().clone(), follow.domain().clone())
            }).collect();
            let node = WotNode::new_list(list.pubkey.clone(), list.alias.clone(), follows);
            node
        }).collect();

        let existing_pubkeys: HashSet<String> = list_nodes.iter().map(|node| node.pubkey.clone()).collect();
        let all_pubkeys: HashSet<String> = self.lists.iter().map(|list| list.get_all_pubkeys()).flatten().collect();

        let diff: Vec<String> = all_pubkeys.difference(&existing_pubkeys).map(|pubkey| pubkey.clone()).collect();
        let mut missing_nodes = diff.into_iter().map(|pubkey| WotNode::new_list(pubkey, "".to_string(), vec![])).collect::<Vec<WotNode>>();
        let mut result = vec![];
        result.append(&mut list_nodes);
        result.append(&mut missing_nodes);
        result
    }

    pub fn get_graph(&self) -> WotGraph {
        let nodes = self.get_follow_nodes();
        WotGraph::new2(nodes)
    }
}


#[cfg(test)]
mod tests {
    use crate::cli::follow_list::{FollowList, Follow};

    use super::WotTransformer;


    // #[test]
    // fn transform_classes() {
    //     let list = FollowList::new_with_follows(
    //         "pk:rcwgkobba4yupekhzxz6imtkyy1ph33emqt16fw6q6cnnbhdoqso".to_string(),
    //         "myList".to_string(),
    //         vec![
    //             Follow::new("pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy".to_string(), 1.0/3.0, None).unwrap(),
    //             Follow::new("pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy".to_string(), -1.0, Some("example.com1".to_string())).unwrap(),
    //             Follow::new("pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy".to_string(), -1.0, Some("example.com1".to_string())).unwrap(),
    //             Follow::new("pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy".to_string(), -1.0, Some("example.com2".to_string())).unwrap()
    //         ],
    //     );

    //     let transformer = WotTransformer::new(vec![list]);
    //     let classes = transformer.get_classes();
    //     assert_eq!(classes.len(), 2);
    //     let first = classes.get(0).unwrap();
    //     assert_eq!(first.pubkey, "pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy".to_string());
    //     let second = classes.get(1).unwrap();
    //     assert_eq!(second.pubkey, "pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy".to_string());
    // }

    #[test]
    fn transform_nodes() {
        let list1 = FollowList::new_with_follows(
            "pk:rcwgkobba4yupekhzxz6imtkyy1ph33emqt16fw6q6cnnbhdoqso".to_string(),
            "me".to_string(),
            vec![
                Follow::new("pk:1bdbmmxenbxuybfai88f1xg1djrpujxix5hw6fh9am7f4x5wapey".to_string(), 1.0, None).unwrap(),
                Follow::new("pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy".to_string(), 1.0/3.0, None).unwrap(),
                Follow::new("pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy".to_string(), -1.0, Some("example.com1".to_string())).unwrap(),
                Follow::new("pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy".to_string(), 1.0/3.0, None).unwrap(),
                Follow::new("pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy".to_string(), -1.0, Some("example.com1".to_string())).unwrap(),
                Follow::new("pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy".to_string(), -1.0, Some("example.com2".to_string())).unwrap()
            ],
        );
        let list2 = FollowList::new_with_follows(
            "pk:1bdbmmxenbxuybfai88f1xg1djrpujxix5hw6fh9am7f4x5wapey".to_string(),
            "Alice".to_string(),
            vec![
                Follow::new("pk:s9y93dtpoibsfcnct35onkeyuiup9dfxwpftgerdqd7u84jcmkfy".to_string(), 1.0/3.0, None).unwrap(),
                Follow::new("pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy".to_string(), -1.0, Some("example.com1".to_string())).unwrap(),
                Follow::new("pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy".to_string(), 1.0/3.0, None).unwrap(),
                Follow::new("pk:rcwgkobba4yupekhzxz6imtkyy1ph33emqt16fw6q6cnnbhdoqso".to_string(), 1.0/3.0, None).unwrap(),
            ],
        );

        let transformer = WotTransformer::new(vec![list1, list2]);
        let nodes = transformer.get_follow_nodes();
        assert_eq!(nodes.len(), 5);
        let first = nodes.get(0).unwrap();
        assert_eq!(first.pubkey, "pk:rcwgkobba4yupekhzxz6imtkyy1ph33emqt16fw6q6cnnbhdoqso".to_string());
        let follows = first.follows.clone();
        assert_eq!(follows.len(), 6);
        assert!(follows[0].attribution.is_none());
        assert_eq!(follows.get(2).unwrap().clone().attribution.unwrap(), "example.com1".to_string());
        let second = nodes.get(1).unwrap();
        assert_eq!(second.pubkey, "pk:1bdbmmxenbxuybfai88f1xg1djrpujxix5hw6fh9am7f4x5wapey".to_string());
    }

    #[test]
    fn pubkey_is_list_and_domain() {
        let list1 = FollowList::new_with_follows(
            "me".to_string(),
            "me".to_string(),
            vec![
                Follow::new("d2".to_string(), 1.0, None).unwrap(),
                Follow::new("d2".to_string(), 0.5, Some("example.com".to_string())).unwrap(),
            ],
        );
        let list2 = FollowList::new_with_follows(
            "d2".to_string(),
            "d2".to_string(),
            vec![
                Follow::new("d1".to_string(), 0.5, Some("example.com".to_string())).unwrap(),
            ],
        );

        let transformer = WotTransformer::new(vec![list1, list2]);
        let graph = transformer.get_graph();

        assert_eq!(graph.nodes.len(), 3);
    }
}