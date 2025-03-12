use std::fmt::Display;

pub enum AxisTransform {
    PolynomialRoot { degree: f64 },
    Log,
    Linear,
}

impl AxisTransform {
    pub fn apply(&self, input: f64) -> f64 {
        match self {
            AxisTransform::PolynomialRoot { degree } => input.powf(1.0 / degree),
            AxisTransform::Log => input.log10(),
            AxisTransform::Linear => input,
        }
    }

    pub fn apply_inverse(&self, input: f64) -> f64 {
        match self {
            AxisTransform::PolynomialRoot { degree } => input.powf(*degree),
            AxisTransform::Log => 10.0f64.powf(input),
            AxisTransform::Linear => input,
        }
    }
}

impl Display for AxisTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AxisTransform::PolynomialRoot { degree } => write!(f, "{degree}-th root"),
            AxisTransform::Log => write!(f, "log10"),
            AxisTransform::Linear => write!(f, "linear"),
        }
    }
}
