use contains::Contains;

/// Histograms count for a range of values which occurred how often.
pub struct Histogram {
    range: (f64, f64),
    edges: Box<[f64]>,
    weights: Box<[u32]>,
}

impl Histogram {
    /// Creates a new histogram with `nbins` bins filling the range
    /// from `low` to `high`.
    pub fn new(nbins: usize, low: f64, high: f64) -> Self {
        let nedges = nbins + 1;
        let mut edges = Vec::with_capacity(nedges);
        let bin_width = (high - low) / (nbins as f64);
        for i in 0..nedges {
            edges.push(low + bin_width * (i as f64));
        }
        // Turn the vectors into boxed slices because we no longe need
        // the `capacity` field.
        Histogram {
            edges: edges.into_boxed_slice(),
            weights: vec![0; nbins].into_boxed_slice(),
            range: (low, high),
        }
    }

    /// Returns the lower and upper limit of the histogram.
    pub fn range(&self) -> &(f64, f64) {
        &self.range
    }

    /// Returns the number of bins of this histogram.
    pub fn num_bins(&self) -> usize {
        self.weights.len()
    }

    /// Returns the number of bin edges of this histogram.
    ///
    /// This is exactly `self.num_bins() + 1`.
    pub fn num_bin_edges(&self) -> usize {
        self.edges.len()
    }

    /// Returns the width of each bin of this histogram.
    pub fn bin_width(&self) -> f64 {
        let &(low, high) = self.range();
        (high - low) / (self.num_bins() as f64)
    }

    /// Returns the low edges of the histogram's bins.
    pub fn bin_low_edges(&self) -> &[f64] {
        &self.edges[..self.num_bins()]
    }

    /// Returns the high edges of the histogram's bins.
    pub fn bin_high_edges(&self) -> &[f64] {
        &self.edges[1..]
    }

    /// Returns the centers of the histogram's bins.
    ///
    /// This method returns an iterator instead of a slice because the
    /// bin centers are calculated on the fly.
    pub fn bin_centers(&self) -> BinCenters {
        BinCenters {
            low_edges: self.bin_low_edges().iter(),
            bin_width: self.bin_width(),
        }
    }

    /// Returns the contents of each of the histogram's bins.
    pub fn bin_contents(&self) -> &[u32] {
        self.weights.as_ref()
    }

    /// Increases the bin located at `x` by one.
    ///
    /// If `x` lies outside of the range of the histogram, this method
    /// does nothing.
    pub fn fill(&mut self, x: f64) {
        self.fill_by(x, 1)
    }

    /// Increases the bin located at `x` by `weight`.
    ///
    /// If `x` lies outside of the range of the histogram, this method
    /// does nothing.
    pub fn fill_by(&mut self, x: f64, weight: u32) {
        if let Some(i) = self.find_bin(x) {
            self.weights[i] += weight;
        }
    }

    /// Returns the index of the bin in which `x` lies.
    ///
    /// If `x` lies outside of the range of this histogram, `None` is
    /// returned.
    pub fn find_bin(&self, x: f64) -> Option<usize> {
        if !self.range.contains(x) {
            return None;
        }
        for (i, bin) in self.edges.windows(2).enumerate() {
            let bin = (bin[0], bin[1]);
            if bin.contains(x) {
                return Some(i);
            }
        }
        unreachable!()
    }
}


/// Iterator over bin centers, returned by `Histogram::bin_centers()`.
pub struct BinCenters<'a> {
    low_edges: ::std::slice::Iter<'a, f64>,
    bin_width: f64,
}

impl<'a> Iterator for BinCenters<'a> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        self.low_edges
            .next()
            .map(|low_edge| low_edge + self.bin_width / 2.0)
    }
}
