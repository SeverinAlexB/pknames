use std::collections::{HashMap, HashSet};

use fancyd_wot::prediction::node::{WotNode, WotFollow};

use super::follow_list::{FollowList, Follow};

struct WotTransformer {
    lists: Vec<FollowList>
}

impl WotTransformer {
    pub fn new(lists: Vec<FollowList>) -> Self {
        WotTransformer{
            lists
        }
    }

    /**
     * Extracts WotNode classes from all lists
     */
    pub fn get_classes(&self) -> Vec<WotNode> {
        let all: Vec<&Follow> = self.lists.iter().map(|list| &list.follows).flatten().collect();
        let domain_follows: Vec<&Follow> = all.iter().filter_map(|follow| {
            match follow.domain() {
                Some(domain) => {
                    Some(*follow)
                },
                None => None
            }

        }).collect();

        let mut map: HashMap<String, Vec<&Follow>> = HashMap::new();
        for follow in domain_follows.into_iter() {
            let target_id = follow.pubkey().clone();
            if !map.contains_key(&target_id) {
                map.insert(target_id.clone(), vec![]);
            };
            let vec = map.get_mut(&target_id).unwrap();
            vec.push(follow);
        };
        let domain_nodes: Vec<WotNode> = map.into_values().into_iter().map(|follows| {
            let claims: HashSet<String> = follows.iter().map(|follow| {
                let op = (*follow).domain().clone();
                let value = op.unwrap();
                value
            }).collect();
            
            WotNode::new_class(follows[0].pubkey().clone(), "".to_string(), Vec::from_iter(claims.into_iter()))
        }).collect();
        domain_nodes
    }


    // /**
    //  * Extracts follow nodes from lists without actual follows
    //  */
    pub fn get_follow_nodes(&self) -> Vec<WotNode> {
        let nodes: Vec<WotNode> = self.lists.iter().map(|list| {
            let follows: Vec<WotFollow> = list.follows.iter().map(|follow| {
                WotFollow::new(list.pubkey.clone(), follow.pubkey().clone(), follow.weight().clone(), follow.domain().clone())
            }).collect();
            let node = WotNode::new_list(list.pubkey.clone(), list.alias.clone(), follows);
            node
        }).collect();
        nodes
    }


}


#[cfg(test)]
mod tests {
    use crate::cli::follow_list::{FollowList, Follow};

    use super::WotTransformer;


    #[test]
    fn transform_classes() {
        let list = FollowList::new_with_follows(
            "pk:rcwgkobba4yupekhzxz6imtkyy1ph33emqt16fw6q6cnnbhdoqso".to_string(),
            "myList".to_string(),
            vec![
                Follow::new("pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy".to_string(), 1.0/3.0, None).unwrap(),
                Follow::new("pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy".to_string(), -1.0, Some("example.com1".to_string())).unwrap(),
                Follow::new("pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy".to_string(), -1.0, Some("example.com1".to_string())).unwrap(),
                Follow::new("pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy".to_string(), -1.0, Some("example.com2".to_string())).unwrap()
            ],
        );

        let transformer = WotTransformer::new(vec![list]);
        let classes = transformer.get_classes();
        assert_eq!(classes.len(), 2);
        let first = classes.get(0).unwrap();
        assert_eq!(first.pubkey, "pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy".to_string());
        assert_eq!(first.get_claims().unwrap().len(), 1);
        let second = classes.get(1).unwrap();
        assert_eq!(second.pubkey, "pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy".to_string());
        assert_eq!(second.get_claims().unwrap().len(), 2);
    }

    #[test]
    fn transform_nodes() {
        let list1 = FollowList::new_with_follows(
            "pk:rcwgkobba4yupekhzxz6imtkyy1ph33emqt16fw6q6cnnbhdoqso".to_string(),
            "myList".to_string(),
            vec![
                Follow::new("pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy".to_string(), 1.0/3.0, None).unwrap(),
                Follow::new("pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy".to_string(), -1.0, Some("example.com1".to_string())).unwrap(),
                Follow::new("pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy".to_string(), 1.0/3.0, None).unwrap(),
                Follow::new("pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy".to_string(), -1.0, Some("example.com1".to_string())).unwrap(),
                Follow::new("pk:1zpo3gfh6657dh8f5rq7z4rzyo3u1tob14r3hcaa6bc9498nbjiy".to_string(), -1.0, Some("example.com2".to_string())).unwrap()
            ],
        );
        let list2 = FollowList::new_with_follows(
            "pk:1bdbmmxenbxuybfai88f1xg1djrpujxix5hw6fh9am7f4x5wapey".to_string(),
            "myList".to_string(),
            vec![
                Follow::new("pk:s9y93dtpoibsfcnct35onkeyuiup9dfxwpftgerdqd7u84jcmkfy".to_string(), 1.0/3.0, None).unwrap(),
                Follow::new("pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy".to_string(), -1.0, Some("example.com1".to_string())).unwrap(),
                Follow::new("pk:kgoxg9i5czhqor1h3b35exfq7hfkpgnycush4n9pab9w3s4a3rjy".to_string(), 1.0/3.0, None).unwrap(),
            ],
        );

        let transformer = WotTransformer::new(vec![list1, list2]);
        let nodes = transformer.get_follow_nodes();
        assert_eq!(nodes.len(), 2);
        let first = nodes.get(0).unwrap();
        assert_eq!(first.pubkey, "pk:rcwgkobba4yupekhzxz6imtkyy1ph33emqt16fw6q6cnnbhdoqso".to_string());
        assert!(first.get_claims().is_none());
        assert_eq!(first.get_follows().unwrap().len(), 5);
        let second = nodes.get(1).unwrap();
        assert_eq!(second.pubkey, "pk:1bdbmmxenbxuybfai88f1xg1djrpujxix5hw6fh9am7f4x5wapey".to_string());
    }
}