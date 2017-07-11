use std::fs::File;
use std::fmt::Debug;
use std::path::Path;

use csv;
use num::Float;
use serde::Deserialize;


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
    pub fn from_file<P>(path: P) -> csv::Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut func = Function::new();
        let mut reader = Self::new_reader(path)?;
        for record in reader.records() {
            let record = record?;
            let point = record.deserialize(None)?;
            func.push(point);
        }
        Ok(func)
    }

    pub fn multiple_from_file<P>(path: P) -> csv::Result<Vec<Self>>
    where
        P: AsRef<Path>,
    {
        let mut funcs = Vec::new();
        let mut reader = Self::new_reader(path)?;

        for record in reader.records() {
            let record = record?;
            if funcs.is_empty() {
                for _ in 1..record.len() {
                    funcs.push(Function::new());
                }
            }
            let record: Vec<F> = record.deserialize(None)?;
            let (&x, rest) = record.split_first().expect("empty record");
            for (&y, func) in rest.iter().zip(&mut funcs) {
                func.push((x, y));
            }
        }
        Ok(funcs)
    }

    pub fn new_reader<P>(path: P) -> csv::Result<csv::Reader<File>>
    where
        P: AsRef<Path>,
    {
        csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .flexible(false)
            .has_headers(true)
            .comment(Some(b'#'))
            .from_path(path)
    }

}
