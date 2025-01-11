

pub struct LSHADE {
    initial_cr: f64,
    initial_fs: f64
}

impl LSHADE {
    pub fn print_info(&self) -> () {
        println!("{}, {}", self.initial_cr, self.initial_fs)
    }

    pub fn new(initial_cr: f64, initial_fs: f64) -> Self {
        LSHADE {
            initial_cr: initial_cr,
            initial_fs: initial_fs
        }
    }

    pub fn set_initial_cr(&mut self, initial_cr: f64) {
        self.initial_cr = initial_cr;
    }
}