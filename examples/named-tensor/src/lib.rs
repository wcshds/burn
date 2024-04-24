use burn::tensor::{backend::Backend, Dim, Distribution, NamedDim, NamedTensor};

NamedDim!(Batch);
NamedDim!(SeqLength);
NamedDim!(DModel);

pub fn run<B: Backend>(device: &B::Device) {
    let batch_size = 32;
    let seq_length = 48;
    let d_model = 24;

    let weights = NamedTensor::<B, (Batch, DModel, DModel)>::random(
        [1, d_model, d_model],
        Distribution::Default,
        device,
    );

    let input = NamedTensor::<B, (Batch, SeqLength, DModel)>::random(
        [batch_size, seq_length, d_model],
        Distribution::Default,
        device,
    );

    // Doesn't compile
    //
    // mismatched types
    //   expected reference `&NamedTensor<B, (Batch, DModel, _)>`
    //   found reference `&NamedTensor<B, (Batch, SeqLength, DModel)>`
    // let output = weights.matmul(&input);

    let output = input.clone().matmul(weights.clone());

    // Doesn't compile
    //
    // mismatched types
    //   expected reference `&NamedTensor<B, (Batch, SeqLength, DModel)>`
    //   found reference `&NamedTensor<B, (Batch, DModel, DModel)>`
    // let output = output.mul(&weights);

    let output = output.mul(input.clone());

    let permut = output.clone().swap_dims::<_, 1, 2>();

    println!("Weights => {weights}");
    println!("Input   => {input}");
    println!("Output  => {output}");
    println!("Permut  => {permut}");
}
