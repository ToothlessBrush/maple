fn main() {
    use maple_engine::prelude::*;
    let node = Empty::builder()
        // Modify the node's initial transform
        .position((10.0, 0.0, 0.0))
        .scale_factor(10.0)
        .build();
}
