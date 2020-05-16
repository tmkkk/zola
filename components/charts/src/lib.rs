use std::collections::HashMap;

use tera::{to_value, from_value, Value, Function as TeraFn, Result};
use config::Config;
use errors::bail;

macro_rules! required_arg {
    ($ty: ty, $e: expr, $err: expr) => {
        match $e {
            Some(v) => match from_value::<$ty>(v.clone()) {
                Ok(u) => u,
                Err(_) => return Err($err.into()),
            },
            None => return Err($err.into()),
        }
    };
}

pub struct FunctionCalculator;

impl FunctionCalculator {
    pub fn new(_config: Config) -> Self {
        FunctionCalculator
    }
}

impl TeraFn for FunctionCalculator {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let formula = required_arg!(
            String,
            args.get("formula"),
            "`get_function_points` require function formula"
        );
        let xrange = required_arg!(
            (f64, f64),
            args.get("xrange"),
            "`get_function_points` require range for x-axis"
        );
        let samples_cnt = required_arg!(
            u32,
            args.get("samples"),
            "`get_function_points` require number of samples/points"
        );

        let expr: meval::Expr = match formula.parse() {
            Ok(x) => x,
            Err(_) => bail!("Could not parse the formula"),
        };
        let func = match expr.bind("x") {
            Ok(f) => f,
            Err(_) => bail!("Could not find the 'x' variable in the formula"),
        };

        let parts = (0..samples_cnt)
            .map(|x| xrange.0 + (xrange.1 - xrange.0) / samples_cnt as f64 * x as f64)
            .filter(|x| func(*x).is_normal())
            .map(|x| format!("{{x:{:.8},y:{:.8}}}", x, func(x)))
            .collect::<Vec<_>>()
            .join(",");

        Ok(to_value(parts).unwrap())
    }
}