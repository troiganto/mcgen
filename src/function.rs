use num::Float;
use csv;
use std::path::Path;
use serde::Deserialize;
use std::fmt::Debug;

#[derive(Debug, Default, Clone)]
pub struct Function<F: Debug + Default + Float> {
    xdata: Vec<F>,
    ydata: Vec<F>,
}

impl<F: Debug + Default + Float> Function<F> {
    pub fn new() -> Self {
        Function::default()
    }

    pub fn xdata(&self) -> &[F] {
        &self.xdata
    }

    pub fn ydata(&self) -> &[F] {
        &self.ydata
    }

    pub fn call(&self, x: F) -> F {
        let some_iend = self.find_first_x_greater_than(x);
        let iend = match some_iend {
            None => panic!("out of range"),
            Some(0) => panic!("out of range"),
            Some(iend) => iend,
        };
        // Apply linear interpolation.
        let (x0, x1) = (self.xdata[iend - 1], self.xdata[iend]);
        let (y0, y1) = (self.ydata[iend - 1], self.ydata[iend]);
        let slope = (y1 - y0) / (x1 - x0);
        (x - x0) * slope + y0
    }

    pub fn push(&mut self, (x, y): (F, F)) {
        if !(x.is_finite() && y.is_finite()) {
            panic!("attempted to add non-finite number");
        }
        if let Some(last_x) = self.xdata.last() {
            if last_x > &x {
                panic!("attempted to build unsorted function");
            }
        }
        self.xdata.push(x);
        self.ydata.push(y);
    }

    pub fn insert(&mut self, x: F, y: F) {
        if !(x.is_finite() && y.is_finite()) {
            panic!("attempted add non-finite number");
        }
        let insert_point = self.find_first_x_greater_than(x);
        if let Some(insert_point) = insert_point {
            // Check for uniqueness.
            if insert_point > 0 && self.xdata[insert_point - 1] == x {
                panic!("attempted to add same point twice");
            }
            self.xdata.insert(insert_point, x);
            self.ydata.insert(insert_point, y);
        } else {
            self.xdata.push(x);
            self.ydata.push(y);
        }
    }

    fn find_first_x_greater_than(&self, x: F) -> Option<usize> {
        for (i, x_item) in self.xdata.iter().enumerate() {
            if *x_item > x {
                return Some(i);
            }
        }
        None
    }
}

impl<F> Function<F>
where
    F: Debug + Default + Float + for<'de> Deserialize<'de>,
{
    pub fn from_file<P>(path: P, delimiter: u8, num_headers: usize) -> csv::Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut func = Function::new();
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .flexible(true)
            .has_headers(false)
            .from_path(path)?;
        for record in reader.records().skip(num_headers) {
            let record = record?;
            let point = record.deserialize(None)?;
            func.push(point);
        }
        Ok(func)
    }
}
