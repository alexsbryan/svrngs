//! The native side of the measure: `derive` over a house's acts through the
//! same `respond` the page calls, median of 20 warm calls.
//!
//!   cargo run --release --example time -- <acts.jsonl> <now>

use std::time::Instant;

// A timer is the adapter that reads the clock (clippy.toml's rule says where).
#[allow(clippy::disallowed_methods)]
fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(path), Some(now)) = (args.next(), args.next()) else {
        eprintln!("usage: time <acts.jsonl> <now>");
        std::process::exit(2);
    };
    let acts = std::fs::read_to_string(&path).expect("the house's acts");
    let now: i64 = now.parse().expect("now, in unix seconds");
    let request = serde_json::json!({ "call": "derive", "acts": acts, "now": now }).to_string();
    let answer = canon_page::respond(request.as_bytes());
    println!(
        "answer: {} bytes, {}",
        answer.len(),
        &answer[..answer.len().min(60)]
    );
    let mut ms: Vec<f64> = (0..21)
        .map(|_| {
            let t = Instant::now();
            canon_page::respond(request.as_bytes());
            t.elapsed().as_secs_f64() * 1000.0
        })
        .skip(1)
        .collect();
    ms.sort_by(f64::total_cmp);
    println!(
        "derive median of 20: native {:.2} ms",
        (ms[9] + ms[10]) / 2.0
    );
}
