use rgb::RGB8;
use terminal_size::{terminal_size, Height, Width};
use textplots::{Chart, ColorPlot, Shape};

pub struct TextPlot {
    train: Vec<(f32, f32)>,
    valid: Vec<(f32, f32)>,
    max_values: usize,
    iteration: usize,
}

impl Default for TextPlot {
    fn default() -> Self {
        Self::new()
    }
}

impl TextPlot {
    pub fn new() -> Self {
        Self {
            train: Vec::new(),
            valid: Vec::new(),
            max_values: 10000,
            iteration: 0,
        }
    }

    pub fn merge(self, other: Self) -> Self {
        let mut other = other;
        let mut train = self.train;
        let mut valid = self.valid;

        train.append(&mut other.train);
        valid.append(&mut other.valid);

        Self {
            train,
            valid,
            max_values: usize::min(self.max_values, other.max_values),
            iteration: self.iteration + other.iteration,
        }
    }

    pub fn update_train(&mut self, item: f32) {
        self.iteration += 1;
        self.train.push((self.iteration as f32, item));

        let x_max = self
            .train
            .last()
            .map(|(iteration, _)| *iteration)
            .unwrap_or(f32::MIN);
        let x_min = self
            .train
            .first()
            .map(|(iteration, _)| *iteration)
            .unwrap_or(f32::MAX);

        if x_max - x_min > self.max_values as f32 && !self.train.is_empty() {
            self.train.remove(0);
        }
    }

    pub fn update_valid(&mut self, item: f32) {
        self.iteration += 1;
        self.valid.push((self.iteration as f32, item));

        let x_max = self
            .valid
            .last()
            .map(|(iteration, _)| *iteration)
            .unwrap_or(f32::MIN);
        let x_min = self
            .valid
            .first()
            .map(|(iteration, _)| *iteration)
            .unwrap_or(f32::MAX);

        if x_max - x_min > self.max_values as f32 && !self.valid.is_empty() {
            self.valid.remove(0);
        }
    }

    pub fn render(&self) -> String {
        let train_color = RGB8::new(255, 140, 140);
        let valid_color = RGB8::new(140, 140, 255);

        let x_max_valid = self
            .valid
            .last()
            .map(|(iteration, _)| *iteration)
            .unwrap_or(f32::MIN);
        let x_max_train = self
            .train
            .last()
            .map(|(iteration, _)| *iteration)
            .unwrap_or(f32::MIN);
        let x_max = f32::max(x_max_train, x_max_valid);

        let x_min_valid = self
            .valid
            .first()
            .map(|(iteration, _)| *iteration)
            .unwrap_or(f32::MAX);
        let x_min_train = self
            .train
            .first()
            .map(|(iteration, _)| *iteration)
            .unwrap_or(f32::MAX);
        let x_min = f32::min(x_min_train, x_min_valid);

        let (width, height) = match terminal_size() {
            Some((Width(w), Height(h))) => (w as u32, u32::min(64, h.into())),
            None => (256, 64),
        };

        Chart::new(width, height, x_min, x_max)
            .linecolorplot(&Shape::Lines(&self.train), train_color)
            .linecolorplot(&Shape::Lines(&self.valid), valid_color)
            .to_string()
    }
}