use chrono::DateTime;
use chrono::Utc;

#[derive(Clone, Copy)]
pub struct Sample {
    pub timestamp: DateTime<Utc>,
    pub value: Option<f64>,
}

pub struct Timeseries {
    buf: Vec<Sample>,
    head: usize,
    len: usize,
    min: Option<f64>,
    max: Option<f64>,
}

impl Timeseries {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);

        Self {
            buf: vec![
                Sample {
                    timestamp: Utc::now(),
                    value: None,
                };
                capacity
            ],
            head: 0,
            len: 0,
            min: None,
            max: None,
        }
    }

    pub fn push(&mut self, timestamp: DateTime<Utc>, value: Option<f64>) {
        let capacity = self.buf.len();

        if self.len < capacity {
            let index = (self.head + self.len) % capacity;

            self.buf[index] = Sample { timestamp, value };

            self.len += 1;
            self.update_min_max(value);
        } else {
            let old = self.buf[self.head].value;

            self.buf[self.head] = Sample { timestamp, value };

            self.head = (self.head + 1) % capacity;

            if old == self.min || old == self.max {
                self.recalculate_min_max();
            }

            self.update_min_max(value);
        }
    }

    fn update_min_max(&mut self, value: Option<f64>) {
        let Some(value) = value else {
            return;
        };

        self.min = Some(match self.min {
            Some(v) if v <= value => v,
            _ => value,
        });

        self.max = Some(match self.max {
            Some(v) if v >= value => v,
            _ => value,
        });
    }

    fn recalculate_min_max(&mut self) {
        self.min = None;
        self.max = None;

        for i in 0..self.len {
            let index = (self.head + i) % self.buf.len();
            let value = self.buf[index].value;

            self.update_min_max(value);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Sample> {
        (0..self.len).map(|i| &self.buf[(self.head + i) % self.buf.len()])
    }

    pub fn newest(&self) -> Option<Sample> {
        if self.len == 0 {
            return None;
        }

        let index = (self.head + self.len - 1) % self.buf.len();
        Some(self.buf[index])
    }

    pub fn oldest(&self) -> Option<Sample> {
        if self.len == 0 {
            return None;
        }

        Some(self.buf[self.head])
    }

    pub fn min(&self) -> Option<f64> {
        self.min
    }

    pub fn max(&self) -> Option<f64> {
        self.max
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == self.buf.len()
    }
}
