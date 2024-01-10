use burn::{
    autodiff::ADBackendDecorator,
    backend::NdArrayBackend,
    module::Module,
    module::Param,
    nn::{loss::CrossEntropyLoss, Linear},
    tensor::{
        activation::{relu, softmax},
        Data, Int, Shape, Tensor,
    },
};

pub type MyBackend = ADBackendDecorator<NdArrayBackend<f32>>;

#[derive(Module, Debug, Clone)]
pub struct FeedForward {
    linears: Vec<Linear<MyBackend>>,
}

impl FeedForward {
    // fn new_linear(inputs: usize, outputs: usize, weights: Vec<f32>) -> Linear<MyBackend> {
    //     let shape = Shape::new([inputs, outputs]);
    //     let data: Data<f32, 2> = Data::new(weights, shape);
    //     Self::new_linear_by_data(data)
    // }

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
        let linears: Vec<Linear<MyBackend>> = weights
            .into_iter()
            .map(|weight| Self::new_linear_by_data(weight))
            .collect();
        Self { linears }
    }

    pub fn to_weights(&self) -> Vec<Data<f32, 2>> {
        let weights: Vec<Data<f32, 2>> = self
            .linears
            .clone()
            .into_iter()
            .map(|linear| linear.weight.to_data())
            .collect();
        weights
    }

    pub fn forward(&self) -> Vec<Vec<f32>> {
        let input = Tensor::ones(Shape::new([1, 1]));
        let mut x: Tensor<MyBackend, 2> = input;
        let mut outputs: Vec<Vec<f32>> = Vec::new();
        for i in 0..(self.linears.len() - 1) {
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
        self.train_verbose(target_index, learning_rates, false)
    }

    pub fn train_verbose(&self, target_index: i64, learning_rates: Vec<f64>, debug_logs: bool) -> Self {
        if learning_rates.len() != self.linears.len() {
            panic!(
                "Number of learning rates different to the number of layers. {} rates vs {} layers",
                learning_rates.len(),
                self.linears.len()
            )
        }

        let input: Tensor<MyBackend, 2> = Tensor::ones(Shape::new([1, 1]));
        let mut x: Tensor<MyBackend, 2> = input;
        for i in 0..(self.linears.len() - 1) {
            x = self.linears[i].forward(x);
            x = relu(x);
        }

        let x = self.linears[self.linears.len() - 1].forward(x);
        // Don't use softmax here because it screws up the loss function.

        let targets: Tensor<MyBackend, 1, Int> =
            Tensor::from_data(Data::new(vec![target_index], Shape::new([1])));
        let loss: Tensor<MyBackend, 1> =
            CrossEntropyLoss::new(None).forward(x.clone(), targets.clone());
        if debug_logs {
            let loss_scalar = loss.to_data().value[0];
            println!("Loss: {}", loss_scalar);
        }

        let mut gradient = loss.backward();

        let new_layers: Vec<Linear<MyBackend>> = self
            .linears
            .iter()
            .enumerate()
            .map(|(i, linear)| {
                let learning_rate = learning_rates[i];
                let grad = linear.weight.grad_remove(&mut gradient).unwrap();
                if debug_logs {
                    println!("Layer {} learning rate {}", i, learning_rate);
                    println!("Gradient: {}", grad.to_data());
                }

                // let grad = grad.div_scalar(loss_scalar);
                // println!("Gradient after loss adjusted: {}", grad.to_data());
                let grad = grad.mul_scalar(learning_rate);
                if debug_logs {
                    println!("Gradient after lr: {}", grad.to_data());
                }
                // Gradient clipping by learning rate.
                // Not sure if the best idea but here we go.
                let grad = grad.clamp(learning_rate * -1.0, learning_rate);
                if debug_logs {
                    println!("Gradient clipped: {}", grad.to_data());
                }
                let grad = grad.to_data();
                let grad: Tensor<MyBackend, 2> = Tensor::from_data(grad);

                let weight = linear.weight.to_data();
                if debug_logs {
                    println!("Old weight {}", weight.clone());
                }
                let weight: Tensor<MyBackend, 2> = Tensor::from_data(weight);

                let new_weight = weight.sub(grad);
                if debug_logs {
                    println!("New weight {}", new_weight.to_data());
                    println!("");
                }
                Self::new_linear_by_data(new_weight.to_data())
            })
            .collect();
        FeedForward {
            linears: new_layers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FeedForward;
    use assert_approx_eq::assert_approx_eq;
    use burn::tensor::{Data, Shape};

    #[test]
    fn run() {
        let weights = vec![
            Data::new(vec![1.0], Shape::new([1, 1])),
            Data::new(vec![1.0, 0.5], Shape::new([1, 2])),
            Data::new(vec![-0.5, 0.0, 1.0, -1.0], Shape::new([2, 2])),
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
    fn train_both_wrong() {
        let weights = vec![
            Data::new(vec![1.0], Shape::new([1, 1])),
            Data::new(vec![1.0, 0.5], Shape::new([1, 2])),
            Data::new(vec![-1.5, 3.0, 0.0, 3.0], Shape::new([2, 2])),
        ];
        let net1: FeedForward = FeedForward::new(weights);
        let out1 = net1.forward();
        assert_approx_eq!(out1[2][0], 0.0, 0.01);
        assert_approx_eq!(out1[2][1], 1.0, 0.01);
        let net2 = net1.train_verbose(0, vec![0.0, 0.1, 3.0], true);
        let out2 = net2.forward();
        println!("out2, {:?}", out2);
        assert_approx_eq!(out2[2][0], 0.7914, 0.001);
        assert_approx_eq!(out2[2][1], 0.2085, 0.001);
    }

    #[test]
    fn train_disagreement() {
        let weights = vec![
            Data::new(vec![1.0], Shape::new([1, 1])),
            Data::new(vec![1.0, 0.5], Shape::new([1, 2])),
            Data::new(vec![-1.0, 3.0, 3.0, 0.0], Shape::new([2, 2])),
        ];
        let net1: FeedForward = FeedForward::new(weights);
        let out1 = net1.forward();
        assert_approx_eq!(out1[2][0], 0.0758, 0.01);
        assert_approx_eq!(out1[2][1], 0.9241418, 0.01);
        let net2 = net1.train(0, vec![0.0, 0.1, 3.0]);
        let out2 = net2.forward();
        assert_approx_eq!(out2[2][0], 0.99226177, 0.001);
        assert_approx_eq!(out2[2][1], 0.007738174, 0.001);
    }
}
