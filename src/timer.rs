use std::time;

pub struct Timer {
    label: String,
    start: time::Instant,
}

impl Timer {
    pub fn with_label(label: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            start: time::Instant::now(),
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let end = time::Instant::now();
        println!(
            ">>> {}: {} seconds<<<",
            self.label,
            (end - self.start).as_secs_f64()
        );
    }
}
