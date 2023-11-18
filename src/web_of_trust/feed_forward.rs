use burn::{
    module::Module,
    module::Param,
    nn::{Linear, loss::CrossEntropyLoss},
    backend::NdArrayBackend,
    tensor::{Tensor, Data, Shape, activation::{relu, softmax}, Int},
    autodiff::ADBackendDecorator
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

    pub fn to_weights(&self) -> Vec<Data<f32, 2>> {
        let weights: Vec<Data<f32, 2>> = self.linears.clone().into_iter().map(|linear| {
            linear.weight.to_data()
        }).collect();
        weights
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

    pub fn train(&self, target_index: i64, learning_rates: Vec<f64>) -> Self {
        if learning_rates.len() != self.linears.len() {
            panic!("Number of learning rates different to the number of layers. {} rates vs {} layers", learning_rates.len(), self.linears.len())
        }


        let input: Tensor<MyBackend, 2> = Tensor::ones(Shape::new([1, 1]));
        let mut x: Tensor<MyBackend, 2> = input;
        for i in 0..(self.linears.len() -1) {
            x = self.linears[i].forward(x);
            x = relu(x);
        }

        let x = self.linears[self.linears.len() - 1].forward(x);
        // Don't use softmax here because it screws up the loss function.

        let targets: Tensor<MyBackend, 1, Int> = Tensor::from_data(Data::new(vec![target_index], Shape::new([1])));
        let loss: Tensor<MyBackend, 1> = CrossEntropyLoss::new(None).forward(x.clone(), targets.clone());
        
        let mut gradient = loss.backward();

        let new_layers: Vec<Linear<MyBackend>> = self.linears.iter().enumerate().map(|(i, linear)| {
            let learning_rate = learning_rates[i];
            let grad = linear.weight.grad_remove(&mut gradient).unwrap();
            let grad = grad.mul_scalar(learning_rate);
            let grad = grad.to_data();
            let grad: Tensor<MyBackend, 2> = Tensor::from_data(grad);

            let weight = linear.weight.to_data();
            let weight:Tensor<MyBackend, 2> = Tensor::from_data(weight);

            let new_weight = weight.sub(grad);
            Self::new_linear_by_data(new_weight.to_data())
        }).collect();
        FeedForward {
            linears: new_layers
        }
    }
}




#[cfg(test)]
mod tests {
    use burn::tensor::{Shape, Data};
    use super::FeedForward;
    use assert_approx_eq::assert_approx_eq;

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
            Data::new(vec![-0.5*3.0, 1.0*3.0, 0.0*3.0, 1.0*3.0], Shape::new([2,2]))
        ];
        let net1: FeedForward = FeedForward::new(weights);
        let out1 = net1.forward();
        assert_approx_eq!(out1[2][0], 0.0, 0.01);
        assert_approx_eq!(out1[2][1], 1.0, 0.01);
        let net2 = net1.train(0, vec![0.0, 0.1, 3.0]);
        let out2 = net2.forward();
        assert_approx_eq!(out2[2][0], 0.6935, 0.001);
        assert_approx_eq!(out2[2][1], 0.3064, 0.001);
    }
}