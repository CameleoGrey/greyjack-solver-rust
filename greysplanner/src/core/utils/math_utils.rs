

pub fn rint(x: f64) -> f64 {
    if (x - x.floor()).abs() < (x.ceil() - x).abs() {x.floor()} else {x.ceil()}
}