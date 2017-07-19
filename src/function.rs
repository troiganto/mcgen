use std::cmp;
use std::iter;
use std::fs::File;
use std::fmt::Debug;
use std::path::Path;
use std::ops::{Add, Sub, Mul, Div, Range};

use csv;
use serde::Deserialize;


pub trait Base: Debug + Copy + PartialOrd {
    fn panicking_cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).expect("not a number")
    }
}

impl<T: Debug + Copy + cmp::PartialOrd> Base for T {}

pub trait Number: Base + Add<Output = Self> + Sub<Output = Self> {}

impl<T: Base + Add<Output = Self> + Sub<Output = Self>> Number for T {}


#[derive(Debug, Clone)]
pub struct Function<X, Y = X>
where
    X: Number,
    Y: Number,
{
    xdata: Vec<X>,
    ydata: Vec<Y>,
    ymin: Y,
    ymax: Y,
}

impl<X: Number, Y: Number> Function<X, Y> {
    pub fn new(x: X, y: Y) -> Self {
        Self::with_capacity(1, x, y)
    }

    pub fn with_capacity(capacity: usize, x: X, y: Y) -> Self {
        let mut xdata = Vec::with_capacity(capacity);
        let mut ydata = Vec::with_capacity(capacity);
        xdata.push(x);
        ydata.push(y);
        Function {
            xdata,
            ydata,
            ymin: y,
            ymax: y,
        }
    }

    pub fn xdata(&self) -> &[X] {
        &self.xdata
    }

    pub fn ydata(&self) -> &[Y] {
        &self.ydata
    }

    pub fn domain(&self) -> Range<X> {
        let start = *self.xdata.first().expect("functions may not be empty");
        let end = *self.xdata.last().expect("functions may not be empty");
        Range { start, end }
    }

    pub fn codomain(&self) -> Range<Y> {
        Range {
            start: *self.min(),
            end: *self.max(),
        }
    }

    pub fn min(&self) -> &Y {
        &self.ymin
    }

    pub fn max(&self) -> &Y {
        &self.ymax
    }

    pub fn push(&mut self, x: X, y: Y) {
        use std::cmp::Ordering::*;

        let last_x = self.domain().end;
        match x.panicking_cmp(&last_x) {
            Less => panic!("point out of order: {:?}", x),
            _ => {},
        }
        if y.panicking_cmp(&self.ymin) == Less {
            self.ymin = y;
        } else if y.panicking_cmp(&self.ymax) == Greater {
            self.ymax = y;
        }
        self.xdata.push(x);
        self.ydata.push(y);
    }
}

impl<X, Y> Function<X, Y>
where
    X: Number,
    Y: Number + Div<X>,
    <Y as Div<X>>::Output: Mul<X, Output = Y>,
{
    pub fn call(&self, x: X) -> Y {
        let iend = match self.xdata.binary_search_by(|x1| x1.panicking_cmp(&x)) {
            Ok(i) => return self.ydata[i],
            Err(i) => i,
        };
        if iend == 0 || iend == self.xdata.len() {
            println!("{:?}", self);
            panic!("out of bounds: {:?}", x)
        }
        let left = (self.xdata[iend - 1], self.ydata[iend - 1]);
        let right = (self.xdata[iend], self.ydata[iend]);
        Self::interpolate(left, right, x)
    }

    fn interpolate((x0, y0): (X, Y), (x1, y1): (X, Y), x: X) -> Y {
        let slope = (y1 - y0) / (x1 - x0);
        y0 + slope * (x - x0)
    }
}

impl<X: Number, Y: Number> iter::Extend<(X, Y)> for Function<X, Y> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (X, Y)>,
    {
        for (x, y) in iter {
            self.push(x, y);
        }
    }
}

impl<X, Y> Function<X, Y>
where
    X: Number + for<'de> Deserialize<'de>,
    Y: Number + for<'de> Deserialize<'de>,
{
    pub fn from_file<P>(path: P) -> csv::Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut reader = Self::new_reader(path)?;
        let mut records = reader.records();

        let mut func = if let Some(record) = records.next() {
            let (x, y) = record?.deserialize(None)?;
            Function::new(x, y)
        } else {
            panic!("empty file");
        };
        for record in records {
            let (x, y) = record?.deserialize(None)?;
            func.push(x, y);
        }
        Ok(func)
    }

    pub fn multiple_from_file<P>(path: P) -> csv::Result<Vec<Self>>
    where
        P: AsRef<Path>,
    {
        let mut reader = Self::new_reader(path)?;
        let mut records = reader.records();

        let mut funcs = if let Some(record) = records.next() {
            let (x, ys): (X, Vec<Y>) = record?.deserialize(None)?;
            ys.into_iter()
                .map(|y| Function::new(x, y))
                .collect::<Vec<_>>()
        } else {
            panic!("empty file");
        };

        for record in records {
            let (x, ys): (X, Vec<Y>) = record?.deserialize(None)?;
            for (y, func) in ys.into_iter().zip(&mut funcs) {
                func.push(x, y);
            }
        }
        Ok(funcs)
    }

    fn new_reader<P>(path: P) -> csv::Result<csv::Reader<File>>
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
