use std::time::{Duration, Instant};

use tch::nn::ModuleT;
use tch::{self, Kind};
use tch::{kind, Tensor};

fn test_simple(vs: &tch::nn::Path) -> tch::nn::Sequential {
    const IMAGE_DIM: i64 = 784;
    const HIDDEN_NODES: i64 = 128;
    const LABELS: i64 = 10;

    tch::nn::seq()
        .add(tch::nn::linear(
            vs / "layer1",
            IMAGE_DIM,
            HIDDEN_NODES,
            Default::default(),
        ))
        .add_fn(|xs| xs.relu())
        .add(tch::nn::linear(
            vs,
            HIDDEN_NODES,
            LABELS,
            Default::default(),
        ))
}

fn speed_test(device: tch::Device) {
    let vs = tch::nn::VarStore::new(device);
    let model = test_simple(&vs.root());

    let start = Instant::now();
    for _ in 0..10000 {
        let __ = model.forward_t(
            &tch::Tensor::rand(&[128, 784], (Kind::Float, device)),
            false,
        );
    }
    let duration: Duration = start.elapsed();
    println!(
        "Time for 1000 iterations {:?} on `{:?}` device",
        duration, device
    );
}

fn grad_example() {
    let mut x = Tensor::from(2.0f32)
        .to_device(tch::Device::Mps)
        .set_requires_grad(true);
    let y = &x * &x + &x + 36;
    println!("y {}", y.double_value(&[]));

    x.zero_grad();
    y.backward();

    let dy_over_dx = x.grad();
    println!("dy/dx {}", dy_over_dx.double_value(&[]))
}

fn test_cpu_and_gpu() {
    let t = Tensor::from_slice(&[3, 1, 4, 1, 5]);
    t.print(); // works on CPU tensors

    println!("t(cpu) {:?}", &t);
    println!("t device: {:?}", &t.device());
    let t = Tensor::randn([5, 4], kind::FLOAT_CPU).to_device(tch::Device::Mps);
    t.print();
    println!("t(mps) {:?}", &t);
    println!("t device: {:?}", &t.device());

    grad_example();

    println!("ran grad example!");
}

fn main() {
    test_cpu_and_gpu();

    speed_test(tch::Device::Mps);
    speed_test(tch::Device::Cpu);
}