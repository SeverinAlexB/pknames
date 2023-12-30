use std::collections::HashMap;

use fancyd_wot::prediction::node::WotNode;

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
     * Extracts WotNode classes from list
     */
    pub fn get_classes(&self, domain_name: &String) -> Vec<WotNode<(), ()>> {
        let all = self.get_all_follows();
        let domain_follows: Vec<&Follow> = all.iter().filter_map(|follow| {
            match follow.domain() {
                Some(domain) => {
                    if domain == domain_name {
                        Some(*follow)
                    } else {
                        None
                    }
                },
                None => None
            }

        }).collect();

        let mut map: HashMap<String, &Follow> = HashMap::new();
        for follow in domain_follows {
            map.insert(follow.target_id(),  follow);
        };
        let domain_nodes: Vec<WotNode<(), ()>> = map.into_values().into_iter().map(|follow| {
            WotNode::new_class(follow.pubkey().clone(), follow.alias().to_string(), ())
        }).collect();
        domain_nodes
    }

    fn get_all_follows(&self) -> Vec<&Follow> {
        let all: Vec<&Follow> = self.lists.iter().map(|list| &list.follows).flatten().collect();
        all
    }

    /**
     * Extracts follow nodes from lists without actual follows
     */
    // pub fn get_follow_nodes(&self) -> Vec<WotNode<(), ()>> {

    //     for list in self.lists.iter() {
    //         let node = WotNode::new_list(list.pubkey.clone(), list.alias.clone(), follows, ());
    //     }

    //     let all: Vec<&Follow> = self.get_all_follows();
    //     let node_follows: Vec<&Follow> = all.into_iter().filter(|follow| {
    //         follow.domain().is_none()
    //     }).collect();

    //     let mut map: HashMap<String, Vec<&Follow>> = HashMap::new();
    //     for follow in node_follows {
    //         if !map.contains_key(&follow.target_id()) {
    //             map.insert(follow.target_id(), vec![]);
    //         };
    //         let mut vec = map.get(&follow.target_id()).unwrap();
    //         vec.push(follow);
    //     };

    // }
}