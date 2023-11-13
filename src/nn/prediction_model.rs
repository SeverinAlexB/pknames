use burn::{
    module::Module,
    module::Param,
    nn::{
        Linear, ReLU, LinearConfig
    },
    backend::NdArrayBackend,
    tensor::{backend::Backend, Tensor, activation::softmax, ops::TensorOps, Data, Shape}
};


#[derive(Module, Debug, Clone)]
pub struct FeedForward {
    linear1: Linear<NdArrayBackend>,
    linear2: Linear<NdArrayBackend>,
    linear3: Linear<NdArrayBackend>,
}





impl FeedForward {
    fn new_linear(inputs: usize, outputs: usize, weights: Vec<f32>) -> Linear<NdArrayBackend> {
        let shape = Shape::new([inputs, outputs]);
        let data: Data<f32, 2> = Data::new(weights, shape);
        let weight: Tensor<NdArrayBackend, 2, _> = Tensor::from_data(data);
        let bias: Option<Param<Tensor<NdArrayBackend, 1>>> = None;
        Linear {
            weight: Param::from(weight),
            bias: bias.map(Param::from),
        }
    }

    pub fn new() -> Self {
        Self {
            linear1: Self::new_linear(1, 1, vec![1.0]),
            linear2: Self::new_linear(1, 2, vec![1.0, 0.5]),
            linear3: Self::new_linear(2, 2, vec![-0.5, 0.0, 1.0, -1.0]),
        }
    }

    pub fn forward<const D: usize>(&self, input: Tensor<NdArrayBackend, D>) -> Tensor<NdArrayBackend, D> {
        let x = self.linear1.forward(input);
        let x = burn::tensor::activation::relu(x);

        let x = self.linear2.forward(x);
        let x = burn::tensor::activation::relu(x);

        let x = self.linear3.forward(x);
        let x = burn::tensor::activation::softmax(x, 1);
        x
    }
}


#[cfg(test)]
mod tests {
    use burn::{tensor::{Tensor, Shape, Data}, backend::NdArrayBackend, module::Param, nn::Linear};

    use super::FeedForward;


    #[test]
    fn run() {        
        let net: FeedForward = FeedForward::new();
        let input = Tensor::ones(Shape::new([1, 1]));
        let output = net.forward(input);
        println!("{}", output)
    }
}