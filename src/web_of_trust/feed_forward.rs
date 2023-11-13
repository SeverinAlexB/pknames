use burn::{
    module::{Module, ADModule},
    module::Param,
    nn::{Linear, loss::CrossEntropyLoss},
    backend::NdArrayBackend,
    tensor::{Tensor, Data, Shape, activation::{relu, softmax}},
    train::ClassificationOutput,
    autodiff::{ADBackendDecorator, grads::Gradients}
};


type MyBackend = ADBackendDecorator<NdArrayBackend<f32>>;

#[derive(Module, Debug, Clone)]
pub struct FeedForward {
    linears: Vec<Linear<MyBackend>>
}


impl FeedForward {

    fn new_linear(inputs: usize, outputs: usize, weights: Vec<f32>) -> Linear<MyBackend> {
        let shape = Shape::new([inputs, outputs]);
        let data: Data<f32, 2> = Data::new(weights, shape);
        Self::new_linear_by_data(data)
    }

    fn new_linear_by_data(weight: Data<f32, 2>) -> Linear<MyBackend> {
        let tensor: Tensor<MyBackend, 2, _> = Tensor::from_data(weight);
        let bias: Option<Param<Tensor<MyBackend, 1>>> = None;
        Linear {
            weight: Param::from(tensor),
            bias: bias.map(Param::from),
        }
    }

        /**
     * Create new network by data (tensor + shape).
     */
    pub fn new(weights: Vec<Data<f32, 2>>) -> Self {
        let linears: Vec<Linear<MyBackend>> = weights.into_iter().map(|weight| Self::new_linear_by_data(weight)).collect();
        Self {
            linears
        }
    }

    pub fn forward(&self) -> Vec<Vec<f32>> {
        let input = Tensor::ones(Shape::new([1, 1]));
        let mut x: Tensor<MyBackend, 2> = input;
        let mut outputs: Vec<Vec<f32>> = Vec::new();
        for i in 0..(self.linears.len() -1) {
            x = self.linears[i].forward(x);
            x = relu(x);
            outputs.push(x.clone().into_data().value);
        }
        // Last output with softmax instead of relu
        let x = self.linears[self.linears.len() - 1].forward(x);
        let x = softmax(x, 1);
        outputs.push(x.clone().into_data().value);
        outputs
    }

    pub fn train(&self, target: Vec<i64>) -> Gradients {
        let input: Tensor<MyBackend, 2> = Tensor::ones(Shape::new([1, 1]));
        let mut x: Tensor<MyBackend, 2> = input;
        for i in 0..(self.linears.len() -1) {
            x = self.linears[i].forward(x);
            x = relu(x);
        }
        // Last output with softmax instead of relu
        let x = self.linears[self.linears.len() - 1].forward(x);
        let x = softmax(x, 1);
        let x_with_batch = x.clone().reshape(Shape::new([1,2]));
        
        let targets: Tensor<MyBackend, 1, _> = Tensor::from_data(Data::new(target, Shape::new([2])));
        let loss: Tensor<MyBackend, 1> = CrossEntropyLoss::new(None).forward(x_with_batch, targets.clone());
        let gradient = loss.backward();
        gradient

        
    }
}




#[cfg(test)]
mod tests {
    use burn::tensor::{Shape, Data};
    use super::FeedForward;

    #[test]
    fn run() {        
        let weights = vec![
            Data::new(vec![1.0], Shape::new([1,1])),
            Data::new(vec![1.0, 0.5], Shape::new([1,2])),
            Data::new(vec![-0.5, 0.0, 1.0, -1.0], Shape::new([2,2]))
        ];
        let net: FeedForward = FeedForward::new(weights);
        let output = net.forward();
        assert_eq!(output[0][0], 1.0);
        assert_eq!(output[1][0], 1.0);
        assert_eq!(output[1][1], 0.5);
        assert_eq!(output[2][0], 0.62245935);
        assert_eq!(output[2][1], 0.37754068);
    }

    #[test]
    fn train() {        
        let weights = vec![
            Data::new(vec![1.0], Shape::new([1,1])),
            Data::new(vec![1.0, 0.5], Shape::new([1,2])),
            Data::new(vec![-0.5, 0.0, 1.0, -1.0], Shape::new([2,2]))
        ];
        let net: FeedForward = FeedForward::new(weights);
        let output = net.train(vec![1,0]);
        let _l = output;
    }
}