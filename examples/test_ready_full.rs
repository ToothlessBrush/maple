// Test that mimics the actual app initialization flow

use maple::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn main() {
    println!("\n=== Testing Ready Event (Full App Flow) ===\n");

    let ready_count = Arc::new(AtomicUsize::new(0));
    let ready_count_clone = ready_count.clone();

    // 1. Scene building (like SceneBuilder::build())
    println!("1. Building scene (nodes go to ready_queue)...");
    let mut scene = Scene::default();
    scene.add(
        "initial_node",
        Empty::builder()
            .on(Ready, move |_ctx: &mut GameContext| {
                let count = ready_count_clone.fetch_add(1, Ordering::SeqCst) + 1;
                println!("   ✓ Ready called on initial_node (call #{})", count);
            })
            .build(),
    );

    // 2. Create context and load scene
    println!("\n2. Creating GameContext and loading scene...");
    let mut ctx = GameContext::new();
    ctx.scene = scene;

    // 3. Simulate app.rs calling emit(Ready) after plugin init
    println!("\n3. Calling emit(Ready) like app.rs does...");
    println!("   (Should NOT process ready_queue to avoid double-Ready)");
    ctx.emit(Ready);

    let count_after_ready = ready_count.load(Ordering::SeqCst);
    println!("   Ready event count: {} (should be 0)", count_after_ready);

    // 4. First Update - this should process the queue
    println!("\n4. First emit(Update) - should process ready_queue:");
    ctx.emit(Update);

    let count_after_update = ready_count.load(Ordering::SeqCst);
    println!("   Ready event count: {} (should be 1)", count_after_update);

    // 5. Add a node dynamically
    let ready_count_clone2 = ready_count.clone();
    println!("\n5. Adding late_node dynamically...");
    ctx.scene.add(
        "late_node",
        Empty::builder()
            .on(Ready, move |_ctx: &mut GameContext| {
                let count = ready_count_clone2.fetch_add(1, Ordering::SeqCst) + 1;
                println!("   ✓ Ready called on late_node (call #{})", count);
            })
            .build(),
    );

    // 6. Next Update should trigger Ready on late node
    println!("\n6. Second emit(Update) - should trigger Ready on late_node:");
    ctx.emit(Update);

    let final_count = ready_count.load(Ordering::SeqCst);
    println!("\n=== Test Results ===");
    println!("Total Ready events: {}", final_count);
    println!("Expected: 2");

    if final_count == 2 && count_after_ready == 0 && count_after_update == 1 {
        println!("\n✓✓✓ TEST PASSED ✓✓✓");
        println!("- Initial emit(Ready) did not trigger Ready (avoided double-Ready)");
        println!("- First Update triggered Ready on initial nodes");
        println!("- Second Update triggered Ready on dynamically added node");
    } else {
        println!("\n✗✗✗ TEST FAILED ✗✗✗");
        println!("After Ready: {} (expected 0)", count_after_ready);
        println!("After Update: {} (expected 1)", count_after_update);
        println!("Final: {} (expected 2)", final_count);
    }
}
