use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let input = line.expect("stdin");
        let input: String = serde_json::from_str(&input).expect("JSON string");
        println!(
            "{}",
            serde_json::to_string(&tezbar_oklch_color_converter::convert(&input)).unwrap()
        );
    }
}
