use std::time::Instant;

use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn main() {
    let val = rand::random::<u8>();


    let start = Instant::now();

    
    let _out = (0..1_000_000_000).into_par_iter()
        .map(|i| {
            i as u8 ^ val
        }).collect::<Vec<_>>();
    
    
    let elapsed = start.elapsed();
    println!(
        "{:.2} messages/sec",
        1_000_000_000 as f64 / elapsed.as_secs_f64()
    );
}