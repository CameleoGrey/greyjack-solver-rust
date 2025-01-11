

pub struct Tabu {
    tabu_size: i64,
    tabu_rate: f64
}

impl Tabu {
    pub fn print_info(&self) -> () {
        println!("{}, {}", self.tabu_size, self.tabu_rate)
    }

    pub fn new(tabu_size: i64, tabu_rate: f64) -> Self {
        Tabu {
            tabu_size: tabu_size,
            tabu_rate: tabu_rate
        }
    }

    pub fn set_tabu_size(&mut self, tabu_size: i64) {
        self.tabu_size = tabu_size;
    }
}
