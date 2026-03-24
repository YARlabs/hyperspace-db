use crate::vector::HyperVector;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

/// `ConeRegion` based on `ConE` (Zhang & Wang, 2021)
/// Cartesian products of 2D angular sectors.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConeRegion {
    pub axes: Vec<f64>,
    pub apertures: Vec<f64>,
    pub cen: f64, // Tolerance / Inside-weight
}

impl ConeRegion {
    pub fn new(axes: Vec<f64>, apertures: Vec<f64>, cen: f64) -> Self {
        Self {
            axes,
            apertures,
            cen,
        }
    }

    pub fn contains<const N: usize>(&self, vector: &HyperVector<N>) -> bool {
        for (d, &entity_axis) in vector.coords.iter().enumerate() {
            if d < self.axes.len() {
                let query_axis = self.axes[d];
                let query_aperture = self.apertures[d];

                // distance_to_axis[i] = |sin((entity_axis[i] - query_axis[i]) / 2)|
                // distance_base[i]    = |sin(query_aperture[i] / 2)|
                let dist_to_axis = ((entity_axis - query_axis) / 2.0).sin().abs();
                let dist_base = (query_aperture / 2.0).sin().abs();

                // Point is inside if distance to axis <= base + tolerance
                if dist_to_axis > dist_base + self.cen {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BallRegion {
    pub center: Vec<f64>,
    pub radius: f64,
}

impl BallRegion {
    pub fn new(center: Vec<f64>, radius: f64) -> Self {
        Self { center, radius }
    }

    pub fn contains<const N: usize>(&self, vector: &HyperVector<N>) -> bool {
        let mut dist_sq = 0.0;
        for (d, coord) in vector.coords.iter().enumerate() {
            if d < self.center.len() {
                let diff = *coord - self.center[d];
                dist_sq += diff * diff;
            }
        }
        dist_sq.sqrt() <= self.radius
    }
}
