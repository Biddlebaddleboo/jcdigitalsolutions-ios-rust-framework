#[cfg(target_os = "ios")]
fn main() {
    let a = std::hint::black_box([1.0_f32, 2.0]);
    let b = std::hint::black_box([3.0_f32, 4.0]);
    let mut output = [0.0_f32; 2];
    let result = ios_accelerate::vector_add(&a, &b, &mut output);
    let _ = result;
    std::hint::black_box(output);
}

#[cfg(not(target_os = "ios"))]
fn main() {}
