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

    #[inline]
    pub fn check_contains(min_bounds: &[f64], max_bounds: &[f64], coords: &[f64]) -> bool {
        for (d, coord) in coords.iter().enumerate() {
            if d < min_bounds.len() && (*coord < min_bounds[d] || *coord > max_bounds[d]) {
                return false;
            }
        }
        true
    }

    pub fn contains(&self, vector: &HyperVector) -> bool {
        Self::check_contains(&self.min_bounds, &self.max_bounds, &vector.coords)
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

    #[inline]
    pub fn check_contains(axes: &[f64], apertures: &[f64], cen: f64, coords: &[f64]) -> bool {
        for (d, &entity_axis) in coords.iter().enumerate() {
            if d < axes.len() {
                let query_axis = axes[d];
                let query_aperture = apertures[d];

                // distance_to_axis[i] = |sin((entity_axis[i] - query_axis[i]) / 2)|
                // distance_base[i]    = |sin(query_aperture[i] / 2)|
                let dist_to_axis = ((entity_axis - query_axis) / 2.0).sin().abs();
                let dist_base = (query_aperture / 2.0).sin().abs();

                // Point is inside if distance to axis <= base + tolerance
                if dist_to_axis > dist_base + cen {
                    return false;
                }
            }
        }
        true
    }

    pub fn contains(&self, vector: &HyperVector) -> bool {
        Self::check_contains(&self.axes, &self.apertures, self.cen, &vector.coords)
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

    #[inline]
    pub fn check_contains(center: &[f64], radius: f64, coords: &[f64]) -> bool {
        if center.len() == 33 {
            if coords.len() < 33 {
                return false;
            }
            let mut inner = -center[0] * coords[0];
            for d in 1..33 {
                inner += center[d] * coords[d];
            }
            let arg = (-inner).max(1.0 + 1e-12);
            arg.acosh() <= radius
        } else {
            let mut dist_sq = 0.0;
            for (d, coord) in coords.iter().enumerate() {
                if d < center.len() {
                    let diff = *coord - center[d];
                    dist_sq += diff * diff;
                }
            }
            dist_sq.sqrt() <= radius
        }
    }

    pub fn contains(&self, vector: &HyperVector) -> bool {
        Self::check_contains(&self.center, self.radius, &vector.coords)
    }
}
