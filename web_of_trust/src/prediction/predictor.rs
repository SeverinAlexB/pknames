use std::{collections::HashMap, fmt};

use burn::tensor::{Data, Shape};

use super::{graph::WotGraph, feed_forward::FeedForward, node::{WotNode, WotFollow}};

#[derive(Clone)]
pub struct WotPredictionResult {
    map: HashMap<String, f32>
}

impl fmt::Display for WotPredictionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.map)
    }
}


impl WotPredictionResult {
    pub fn new() -> Self {
        WotPredictionResult { map: HashMap::new() }
    }

    pub fn get_pubkeys(&self) -> Vec<&String> {
        let keys: Vec<&String> = self.map.keys().collect();
        keys
    }

    pub fn get_value(&self, pubkey: &str) -> Option<f32> {
        let option = self.map.get(pubkey);
        match option {
            None => None,
            Some(val) => {
                Some(val.clone())
            }
        }
    }
}


pub struct WotPredictor {
    pub graph: WotGraph
}

impl WotPredictor {

    /**
     * Predict the probability of the classes.
     */
    pub fn predict(&self) -> WotPredictionResult {
        let weights = self.get_ff_weights();
        let feed_forward = FeedForward::new(weights);
        let _prediction = feed_forward.forward();
        let layers = self.layers_with_temp_nodes();

        let mut map = HashMap::new();
        layers.iter().enumerate().for_each(|(i, layer)| {
            layer.iter().enumerate().for_each(|(j, node)| {
                let power = _prediction[i][j];
                map.insert(node.pubkey.clone(), power);
            });
        });

        WotPredictionResult { map: map }
    }

    pub fn train(&mut self, correct_pubkey: &str, learning_rates: Vec<f64>) {
        let weights = self.get_ff_weights();
        let feed_forward = FeedForward::new(weights);
        let classes = self.graph.get_classes();
        let found = classes.iter().enumerate().find(|(_, class)| {
            class.pubkey == correct_pubkey
        });
        let target_index = match found {
            None => {
                panic!("Can't train graph on non-existing public key {}", correct_pubkey);
            },
            Some((i, _)) => {
                i
            }
        };



        
        // Set learning rate of me layer to 0.
        let mut lrs = vec![0.0];
        lrs.extend(learning_rates);

        // Multiply class lr by 3 to correct for the expansion of the weight range.
        // let class_lr = lrs[lrs.len()-1] * 3.0;
        let count = lrs.len();
        for (i, w) in lrs.iter_mut().enumerate() {
            if i == count -1 {
                *w *= 3.0;
            }
        }

        let trained = feed_forward.train(target_index as i64, lrs);
        let weights = trained.to_weights();
        self.set_ff_weights(weights);
    }

    fn layers_with_temp_nodes(&self) -> Vec<Vec<WotNode>> {
        let mut layers: Vec<Vec<WotNode>> = self.graph.get_layers().iter().map(|layer| {
            let new_layer : Vec<WotNode> = layer.iter().map(|node|  {
                let cloned = (*node).clone();
                cloned
            }).collect();
            new_layer
        }).collect();
        for i in 1..layers.len() {
            let previous_layer = layers[i-1].clone();
            let current_layer = &mut layers[i];
            let mut current_layer_map = HashMap::new();
            for node in current_layer.iter() {
                current_layer_map.insert(node.pubkey.clone(), node.clone());
            };

            for previous_node in previous_layer.iter() {
                for follow in previous_node.follows.iter() {
                    let target_node_in_next_layer = current_layer_map.get(&follow.target_pubkey);
                    if let None = target_node_in_next_layer {
                        let temp = WotNode{
                            pubkey: follow.target_pubkey.clone(),
                            alias: "".to_string(),
                            follows:  vec![WotFollow::new(follow.target_pubkey.clone(), follow.target_pubkey.clone(), 1.0, None)]
                        };
                        current_layer.push(temp);
                    }
                };
            };

            current_layer.sort_unstable_by_key(|node| node.pubkey.clone());
        };
        layers
    }

    fn two_layers_to_weights(&self, previous_layer: &Vec<WotNode>, current_layer: &Vec<WotNode>) -> Data<f32, 2> {
        let is_last_layer = current_layer[0].follows.len() == 0;
        // let is_last_layer = if let WotNodeType::WotClass{..} = current_layer[0].clone().typ {
        //     true
        // } else {
        //     false
        // };
        let weights: Vec<Vec<f32>> = previous_layer.iter().map(|previous| {
            current_layer.iter().map(|current| {
                let follow = previous.get_follow(&current.pubkey);
                match follow {
                    None => {
                        0.0
                    },
                    Some(follow) => {
                        if is_last_layer {
                            follow.weight * 3.0
                        } else {
                            follow.weight
                        }
                    }
                }
            }).collect()
        }).collect();
        let weights = weights.concat();
        let data = Data::new(weights, Shape::new([previous_layer.len(), current_layer.len()]));
        data
    }

    fn get_ff_weights(&self) -> Vec<Data<f32, 2>> {
        let layers = self.layers_with_temp_nodes();
        let mut weights: Vec<Data<f32, 2>> = vec![Data::new(vec![1.0], Shape::new([1,1]))];
        for i in 1..layers.len() {
            let previous_layer = &layers[i -1];
            let current_layer = &layers[i];
            let weight = self.two_layers_to_weights(&previous_layer, &current_layer);
            weights.push(weight);
        }
        weights
    }

    fn set_ff_weights(&mut self, all_weights: Vec<Data<f32, 2>>) -> () {
        let layers = self.layers_with_temp_nodes();
        for i in 1..all_weights.len() {
            let previous_layer = &layers[i -1];
            let current_layer = &layers[i];
            let weights = &all_weights[i];

            for x in 0..previous_layer.len() {
                let previous_node = &previous_layer[x];
                for y in 0..current_layer.len() {
                    let current_node = &current_layer[y];
                    let is_last_layer = current_node.follows.len() == 0;
                    // let is_last_layer = match &current_node.typ {
                    //     WotNodeType::WotClass{..} => {
                    //         true
                    //     },
                    //     _ => false
                    // };

                    let index = x*current_layer.len() + y;
                    let mut weight = weights.value[index];
                    let follow = self.graph.get_follow_mut(&previous_node.pubkey, &current_node.pubkey);
                    if let Some(follow) = follow {
                        if is_last_layer {
                            weight = weight / 3.0; // Divide by 3 to put it back into the -1 to 1 range.
                        };
                        let weight = weight.min(1.0).max(-1.0); // Fix it in the -1 to +1 range in case the gradient got out of hand.
                        follow.weight = weight;
                    };
                }
            }
        }
        
    }

}

impl From<WotGraph> for WotPredictor {
    fn from(value: WotGraph) -> Self {
        WotPredictor { graph: value }
    }
}


#[cfg(test)]
mod tests {


    use crate::prediction::predictor::WotPredictor;

    use super::super::node::{WotNode, WotFollow};
    use super::WotGraph;
    use assert_approx_eq::assert_approx_eq;

    /**
     * Constructs a simple graph
     */
    fn get_simple_graph() -> WotGraph {
        let mut nodes: Vec<WotNode> = Vec::new();

        // Classes
        nodes.push(WotNode {
            pubkey: "d1".to_string(),
            alias: String::from("example.com1"),
            follows: vec![],
        });
        nodes.push(WotNode {
            pubkey: "d2".to_string(),
            alias: String::from("example.com2"),
            follows: vec![],
        });

        nodes.push(WotNode {
            pubkey: "n2".to_string(),
            alias: "".to_string(),
            follows: vec![
                    WotFollow::new("n2".to_string(), "d1".to_string(), 1.0, Some("example.com".to_string())),
                    WotFollow::new("n2".to_string(), "d2".to_string(), -1.0, Some("example.com".to_string()))
                ]
        });

        nodes.push(WotNode {
            pubkey: "n1".to_string(),
            alias: "".to_string(),
            follows: vec![
                    WotFollow::new("n1".to_string(), "d1".to_string(), -0.5, Some("example.com".to_string())),
                    WotFollow::new("n1".to_string(), "d2".to_string(), 0.0, Some("example.com".to_string()))
                ]
        });

        nodes.push(WotNode {
            pubkey: "me".to_string(),
            alias: "".to_string(),
            follows: vec![
                    WotFollow::new("me".to_string(), "n1".to_string(), 1.0, None),
                    WotFollow::new("me".to_string(), "n2".to_string(), 0.5, None)
                ]
        });

        WotGraph::new(nodes)
    }

    fn get_complex_graph() -> WotGraph {
        let mut nodes: Vec<WotNode> = Vec::new();

        // Classes
        nodes.push(WotNode {
            pubkey: "d1".to_string(),
            alias: String::from("example.com1"),
            follows: vec![],
        });
        nodes.push(WotNode {
            pubkey: "d2".to_string(),
            alias: String::from("example.com2"),
            follows: vec![],
        });

        nodes.push(WotNode {
            pubkey: "n3".to_string(),
            alias: "".to_string(),
            follows: vec![
                WotFollow::new("n1".to_string(), "d1".to_string(), 1.0, Some("example.com".to_string())),
                WotFollow::new("n1".to_string(), "d2".to_string(), 1.0, Some("example.com".to_string()))
            ]
        });

        nodes.push(WotNode {
            pubkey: "n2".to_string(),
            alias: "".to_string(),
            follows: vec![
                WotFollow::new("n2".to_string(), "d2".to_string(), -1.0, Some("example.com".to_string()))
            ]
        });

        nodes.push(WotNode {
            pubkey: "n1".to_string(),
            alias: "".to_string(),
            follows: vec![
                WotFollow::new("n1".to_string(), "n3".to_string(), 1.0, None),
            ]
        });

        nodes.push(WotNode {
            pubkey: "me".to_string(),
            alias: "".to_string(),
            follows: vec![
                WotFollow::new("me".to_string(), "n1".to_string(), 1.0, None),
                WotFollow::new("me".to_string(), "n2".to_string(), 0.5, None)
            ]
        });

        WotGraph::new(nodes)
    }

    #[test]
    fn from_into_graph() {
        let old_graph: WotGraph = get_simple_graph();
        let predictor: WotPredictor = old_graph.clone().into();
        let new_graph: WotGraph = predictor.into();

        assert_eq!(old_graph.depth(), new_graph.depth());
        assert_eq!(old_graph.nodes.len(), new_graph.nodes.len());
    }

    #[test]
    fn temp_nodes_layers() {
        let graph = get_complex_graph();
        let predictor: WotPredictor = graph.into();

        let layers = predictor.layers_with_temp_nodes();
        assert_eq!(layers.len(), 4);
        assert_eq!(layers[0].len(), 1);
        assert_eq!(layers[1].len(), 2);
        assert_eq!(layers[2].len(), 2);
        assert_eq!(layers[3].len(), 2);
    }

    #[test]
    fn predict_simple() {
        let graph = get_simple_graph();
        let predictor: WotPredictor = graph.into();
        let result = predictor.predict();
        assert_eq!(result.get_value("d1").unwrap(), 0.81757444);
        assert_eq!(result.get_value("d2").unwrap(), 0.18242551);
    }

    #[test]
    fn predict_complex() {
        let graph = get_complex_graph();
        let predictor: WotPredictor = graph.into();
        let result = predictor.predict();

        assert_eq!(result.get_value("me").unwrap(), 1.0);
        assert_eq!(result.get_value("n1").unwrap(), 1.0);
        assert_eq!(result.get_value("n2").unwrap(), 0.5);
        assert_eq!(result.get_value("n3").unwrap(), 1.0);
        assert_eq!(result.get_value("d1").unwrap(), 0.81757444);
        assert_eq!(result.get_value("d2").unwrap(), 0.18242551);
    }

    #[test]
    fn train_simple() {
        let graph = get_simple_graph();
        let mut predictor: WotPredictor = graph.into();
        predictor.train("d2", vec![0.1, 1.0]);
        let new_weights = predictor.get_ff_weights();
        assert_approx_eq!(new_weights[1].value[0], 1.0, 0.1);
        assert_approx_eq!(new_weights[1].value[1], 0.4, 0.1);
        assert_approx_eq!(new_weights[2].value[0], -3.0, 0.1);
        assert_approx_eq!(new_weights[2].value[1], 2.45, 0.1);
        assert_approx_eq!(new_weights[2].value[2], 1.77, 0.1);
        assert_approx_eq!(new_weights[2].value[3], -1.77, 0.1);
    }

    #[test]
    fn train_back_to_graph() {
        let graph = get_simple_graph();
        let mut predictor: WotPredictor = graph.into();
        predictor.train("d2", vec![0.1, 1.0]);
        let updated: WotGraph = predictor.into();
        
    }
}
