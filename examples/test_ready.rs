// Test to verify Ready event fires for nodes added after scene initialization

use maple::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn main() {
    println!("\n=== Testing Ready Event Behavior ===\n");

    // Counter to track Ready event calls
    let ready_count = Arc::new(AtomicUsize::new(0));
    let ready_count_clone = ready_count.clone();

    // Create a scene with initial node
    let mut scene = Scene::default();

    println!("1. Adding initial node to scene (should go to ready_queue)...");
    scene.add(
        "initial_node",
        Empty::builder()
            .on(Ready, move |_ctx: &mut GameContext| {
                let count = ready_count_clone.fetch_add(1, Ordering::SeqCst) + 1;
                println!("   ✓ Ready called on initial_node (call #{})", count);
            })
            .build(),
    );

    // Simulate what happens during app initialization
    println!("\n2. Creating GameContext...");
    let mut ctx = GameContext::new();
    ctx.scene = scene;

    // First emit - should process ready_queue and trigger Ready on initial node
    println!("\n3. First emit(Update) - should process ready_queue:");
    println!("   - Emit Ready to nodes in queue");
    println!("   - Move nodes from queue to main tree");
    println!("   - Emit Update to main tree");
    ctx.emit(Update);

    let count_after_first = ready_count.load(Ordering::SeqCst);
    println!("\n   Ready event count: {}", count_after_first);

    // Now add a node after the scene is "running"
    let ready_count_clone2 = ready_count.clone();
    println!("\n4. Adding late_node AFTER scene is running...");
    println!("   (This is the scenario that was broken before)");
    ctx.scene.add(
        "late_node",
        Empty::builder()
            .on(Ready, move |_ctx: &mut GameContext| {
                let count = ready_count_clone2.fetch_add(1, Ordering::SeqCst) + 1;
                println!("   ✓ Ready called on late_node (call #{})", count);
            })
            .build(),
    );

    // Next emit should trigger Ready on the late node
    println!("\n5. Second emit(Update) - should trigger Ready on late_node:");
    ctx.emit(Update);

    let final_count = ready_count.load(Ordering::SeqCst);
    println!("\n=== Test Results ===");
    println!("Total Ready events fired: {}", final_count);
    println!("Expected: 2 (one for initial_node, one for late_node)");

    if final_count == 2 {
        println!("\n✓✓✓ TEST PASSED ✓✓✓");
        println!("Both nodes received Ready event exactly once!");
    } else {
        println!("\n✗✗✗ TEST FAILED ✗✗✗");
        println!("Expected 2 Ready events, got {}", final_count);
    }
}
