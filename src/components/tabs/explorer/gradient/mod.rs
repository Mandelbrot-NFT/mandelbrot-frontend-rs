pub mod step;
pub mod wave;

fn smoothstep(a: &[u8; 3], b: &[u8; 3], t: f64) -> [u8; 3] {
    let st = t * t * (3.0 - 2.0 * t);
    (0..3)
        .map(|i| ((1.0 - st) * a[i] as f64 + st * b[i] as f64).round() as u8)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
