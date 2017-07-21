use std::cmp;
use std::iter;
use std::fs::File;
use std::fmt::Debug;
use std::path::Path;
use std::ops::{Add, Sub, Mul, Div, Range};

use csv;
use serde::Deserialize;


/// A trait alias that simplifies the signature of `Number`.
///
/// The bounds are necessary to auto-derive `Clone` and `Debug`.
///
/// Furthermore, we require the numbers to be `PartialOrd` and then
/// implement a comparison function that panics on uncomparable
/// arguments. This allows us to more easily use the `Function` type
/// with floats without using a wrapper type to implement `Ord`.
pub trait Primitive: Debug + Copy + PartialOrd {
    fn panicking_cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).expect("not a number")
    }
}

impl<T: Debug + Copy + cmp::PartialOrd> Primitive for T {}


/// The trait of all types that can be used with `Function`.
///
/// The essential operations are addition and subtraction. Furthermore,
/// `Function::call` requires that multiplication and division are
/// implemented in a sensible manner.
pub trait Number: Primitive + Add<Output = Self> + Sub<Output = Self> {}

impl<T: Primitive + Add<Output = Self> + Sub<Output = Self>> Number for T {}


/// Type that implements piecewise linear functions.
///
/// This type allows reading CSV files as function definitions and then
/// evaluating the function via linear interpolation.
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
    /// Creates a function that initially contains only one point.
    pub fn new(x: X, y: Y) -> Self {
        Function {
            xdata: vec![x],
            ydata: vec![y],
            ymin: y,
            ymax: y,
        }
    }

    /// Creates a function that initially contains only one point.
    ///
    /// Additionally, the contained vectors are initialized with the
    /// given capacity.
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

    /// Returns a reference to the X-axis portions of the points.
    pub fn xdata(&self) -> &[X] {
        &self.xdata
    }

    /// Returns a reference to the Y-axis portions of the points.
    pub fn ydata(&self) -> &[Y] {
        &self.ydata
    }

    /// Returns the range of allowed X-values to `call` the function
    /// with.
    pub fn domain(&self) -> Range<X> {
        let start = *self.xdata.first().expect("functions may not be empty");
        let end = *self.xdata.last().expect("functions may not be empty");
        Range { start, end }
    }

    /// Returns the range of possible Y-values returned by this
    /// function.
    pub fn codomain(&self) -> Range<Y> {
        Range {
            start: *self.min(),
            end: *self.max(),
        }
    }

    /// Returns the minimum of the function.
    pub fn min(&self) -> &Y {
        &self.ymin
    }

    /// Returns the maximum of the function.
    pub fn max(&self) -> &Y {
        &self.ymax
    }

    /// Adds another point to the function.
    ///
    /// The function can only be extended to the right.
    ///
    /// Note that it is technically allowed to pass in the same `x`
    /// twice with different Y-values. The function then becomes
    /// indeterminate at this point.
    ///
    /// # Panics
    /// This panics if `x` is less than the last X-value added.
    /// It also panics if `x` or `y` is not comparable to other values;
    /// for example by being NaN.
    pub fn push(&mut self, x: X, y: Y) {
        use std::cmp::Ordering::*;

        let last_x = self.domain().end;
        match X::panicking_cmp(&x, &last_x) {
            Less => panic!("point out of order: {:?}", x),
            _ => {},
        }
        if Y::panicking_cmp(&y, &self.ymin) == Less {
            self.ymin = y;
        } else if Y::panicking_cmp(&y, &self.ymax) == Greater {
            self.ymax = y;
        }
        self.xdata.push(x);
        self.ydata.push(y);
    }

    /// Scales the X-axis with a constant factor.
    ///
    /// # Panics
    /// This panics if after the transformation, the X-values are no
    /// longer sorted in an increasing manner. This happens, for
    /// example, by passing the factor `-1.0`.
    pub fn scale_x<S>(self, scale: S) -> Function<S::Output, Y>
    where
        S: Copy + Mul<X>,
        S::Output: Number,
    {
        let xdata = self.xdata
            .into_iter()
            .map(|x| scale * x)
            .collect::<Vec<_>>();
        if !is_sorted(&xdata) {
            panic!("xdata is out of order");
        }
        Function {
            xdata,
            ydata: self.ydata,
            ymin: self.ymin,
            ymax: self.ymax,
        }
    }

    /// Scales the Y-axis with a constant factor.
    ///
    /// This has to iterate over the Y-axis data multiple times to
    /// recalculate the minimum and maximum.
    pub fn scale_y<S>(self, scale: S) -> Function<X, S::Output>
    where
        S: Copy + Mul<Y>,
        S::Output: Number,
    {
        let ydata = self.ydata
            .into_iter()
            .map(|y| scale * y)
            .collect::<Vec<_>>();
        let ymin = *ydata
                        .iter()
                        .min_by(|left, right| left.panicking_cmp(right))
                        .expect("missing minimum");
        let ymax = *ydata
                        .iter()
                        .max_by(|left, right| left.panicking_cmp(right))
                        .expect("missing maximum");
        Function {
            xdata: self.xdata,
            ydata,
            ymin,
            ymax,
        }
    }

    /// Scales both the X- and the Y-axis by constant factors.
    pub fn scale<S, T>(self, xscale: S, yscale: T) -> Function<S::Output, T::Output>
    where
        S: Copy + Mul<X>,
        S::Output: Number,
        T: Copy + Mul<Y>,
        T::Output: Number,
    {
        self.scale_x(xscale).scale_y(yscale)
    }
}

impl<X, Y> Function<X, Y>
where
    X: Number,
    Y: Number + Div<X>,
    <Y as Div<X>>::Output: Mul<X, Output = Y>,
{
    /// Evaluates the function at a given point.
    ///
    /// If `x` is exactly the value of one of points in the function,
    /// the corresponding Y-value is returned.
    /// Otherwise, an interpolated value between the two closest points
    /// is returned.
    ///
    /// # Panics
    /// This panics if `x` lies not within the domain of this function.
    pub fn call(&self, x: X) -> Y {
        let iend = match self.xdata.binary_search_by(|x1| x1.panicking_cmp(&x)) {
            Ok(i) => return self.ydata[i],
            Err(i) => i,
        };
        if iend == 0 || iend == self.xdata.len() {
            panic!("out of bounds: {:?}", x)
        }
        let left = (self.xdata[iend - 1], self.ydata[iend - 1]);
        let right = (self.xdata[iend], self.ydata[iend]);
        Self::interpolate(left, right, x)
    }

    /// Interpolate between two points.
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
    /// Reads a function from a CSV file.
    ///
    /// The CSV file must have two columns, separated by tab characters
    /// (`'\t'`). It must have a header line and may contain comment
    /// lines starting with `'#'`.
    ///
    /// # Errors
    /// This function fails if the file cannot be read or any number
    /// cannot be parsed.
    ///
    /// # Panics
    /// This panics if any number gets parsed as NaN.
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

    /// Reads several functions from a CSV file.
    ///
    /// The CSV file must have at least two columns, separated by tab
    /// characters (`'\t'`). It must have a header line and may contain
    /// comment lines starting with `'#'`.
    ///
    /// The first column is used for the X-values. Each further column
    /// defines the Y-values of another function. For example, a
    /// CSV file with four columns creates three functions.
    ///
    /// # Errors
    /// This function fails if the file cannot be read or any number
    /// cannot be parsed.
    ///
    /// # Panics
    /// This panics if any number gets parsed as NaN.
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

    /// Creates a common reader for both `from_file()` functions.
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


/// Returns `true` if all numbers are sorted in an increasing manner.
///
/// # Panics
/// This panics if any number is not comparable to its neighbors.
fn is_sorted<X: Number>(nums: &[X]) -> bool {
    use std::cmp::Ordering::Greater;

    nums.windows(2)
        .all(|pair| X::panicking_cmp(&pair[0], &pair[1]) != Greater)
}
