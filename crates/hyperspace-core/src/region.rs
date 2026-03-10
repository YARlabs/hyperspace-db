use crate::vector::HyperVector;

#[derive(Debug, Clone)]
pub struct BoxRegion {
    pub min_bounds: Vec<f64>,
    pub max_bounds: Vec<f64>,
}

impl BoxRegion {
    pub fn new(min_bounds: Vec<f64>, max_bounds: Vec<f64>) -> Self {
        Self {
            min_bounds,
            max_bounds,
        }
    }

    pub fn contains<const N: usize>(&self, vector: &HyperVector<N>) -> bool {
        for (d, coord) in vector.coords.iter().enumerate() {
            if d < self.min_bounds.len()
                && (*coord < self.min_bounds[d] || *coord > self.max_bounds[d])
            {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct ConeRegion {
    pub axes: Vec<f64>,
    pub apertures: Vec<f64>,
    pub cen: f64,
    dist_base: Vec<f64>,
}

impl ConeRegion {
    pub fn new(axes: Vec<f64>, apertures: Vec<f64>, cen: f64) -> Self {
        let mut dist_base = Vec::with_capacity(apertures.len());
        for &ap in &apertures {
            dist_base.push((ap / 2.0).sin().abs());
        }
        Self {
            axes,
            apertures,
            cen,
            dist_base,
        }
    }

    pub fn contains<const N: usize>(&self, vector: &HyperVector<N>) -> bool {
        for (d, coord) in vector.coords.iter().enumerate() {
            if d < self.axes.len() {
                let dist_axis = ((coord - self.axes[d]) / 2.0).sin().abs();
                if dist_axis > self.dist_base[d] + self.cen {
                    return false;
                }
            }
        }
        true
    }
}
