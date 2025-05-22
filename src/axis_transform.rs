use std::fmt::Display;

pub enum AxisTransform {
    PolynomialRoot { degree: f64 },
}

impl AxisTransform {
    pub fn apply(&self, input: f64) -> f64 {
        match self {
            AxisTransform::PolynomialRoot { degree } => input.powf(1.0 / degree),
        }
    }

    pub fn apply_inverse(&self, input: f64) -> f64 {
        match self {
            AxisTransform::PolynomialRoot { degree } => input.powf(*degree),
        }
    }
}

impl Display for AxisTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AxisTransform::PolynomialRoot { degree } => write!(f, "{degree}-th root"),
        }
    }
}
