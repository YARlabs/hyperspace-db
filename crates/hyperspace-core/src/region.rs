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

    #[inline]
    pub fn contains_coords(&self, coords: &[f64]) -> bool {
        Self::check_contains(&self.min_bounds, &self.max_bounds, coords)
    }

    pub fn contains(&self, vector: &HyperVector) -> bool {
        self.contains_coords(&vector.coords)
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
        thread_local! {
            static THRESHOLD_CACHE: std::cell::RefCell<(usize, usize, u64, Vec<f64>)> =
                const { std::cell::RefCell::new((0, 0, 0, Vec::new())) };
        }

        let axes_ptr = axes.as_ptr() as usize;
        let ap_ptr = apertures.as_ptr() as usize;
        let cen_bits = cen.to_bits();

        THRESHOLD_CACHE.with(|cell| {
            let mut cache = cell.borrow_mut();
            if cache.0 != axes_ptr
                || cache.1 != ap_ptr
                || cache.2 != cen_bits
                || cache.3.len() != apertures.len()
            {
                cache.0 = axes_ptr;
                cache.1 = ap_ptr;
                cache.2 = cen_bits;
                cache.3.clear();
                cache.3.reserve(apertures.len());
                for &ap in apertures {
                    cache.3.push((ap * 0.5).sin().abs() + cen);
                }
            }

            let thresholds = &cache.3;
            let n = axes.len().min(coords.len()).min(thresholds.len());
            for d in 0..n {
                let dist_to_axis = ((coords[d] - axes[d]) * 0.5).sin().abs();
                if dist_to_axis > thresholds[d] {
                    return false;
                }
            }
            true
        })
    }

    #[inline]
    pub fn contains_coords(&self, coords: &[f64]) -> bool {
        Self::check_contains(&self.axes, &self.apertures, self.cen, coords)
    }

    pub fn contains(&self, vector: &HyperVector) -> bool {
        self.contains_coords(&vector.coords)
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
        if radius < 0.0 {
            return false;
        }
        if center.len() == 33 {
            if coords.len() < 33 {
                return false;
            }
            let max_cosh = radius.cosh();
            let mut inner = -center[0] * coords[0];
            let mut acc = [0.0; 4];
            let (chunks_c, rem_c) = center[1..33].as_chunks::<4>();
            let (chunks_x, rem_x) = coords[1..33].as_chunks::<4>();
            for (ca, cx) in chunks_c.iter().zip(chunks_x.iter()) {
                for l in 0..4 {
                    acc[l] += ca[l] * cx[l];
                }
            }
            inner += acc[0] + acc[1] + acc[2] + acc[3];
            for (&c, &x) in rem_c.iter().zip(rem_x.iter()) {
                inner += c * x;
            }
            let arg = (-inner).max(1.0 + 1e-12);
            arg <= max_cosh
        } else {
            let max_dist_sq = radius * radius;
            let n = center.len().min(coords.len());
            let mut acc = [0.0; 4];
            let (chunks_c, rem_c) = center[..n].as_chunks::<4>();
            let (chunks_x, rem_x) = coords[..n].as_chunks::<4>();
            for (ca, cx) in chunks_c.iter().zip(chunks_x.iter()) {
                for l in 0..4 {
                    let diff = cx[l] - ca[l];
                    acc[l] += diff * diff;
                }
            }
            let mut dist_sq = acc[0] + acc[1] + acc[2] + acc[3];
            for (&c, &x) in rem_c.iter().zip(rem_x.iter()) {
                let diff = x - c;
                dist_sq += diff * diff;
            }
            dist_sq <= max_dist_sq
        }
    }

    #[inline]
    pub fn contains_coords(&self, coords: &[f64]) -> bool {
        Self::check_contains(&self.center, self.radius, coords)
    }

    pub fn contains(&self, vector: &HyperVector) -> bool {
        self.contains_coords(&vector.coords)
    }
}
