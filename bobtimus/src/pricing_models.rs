use core::f64;
use nalgebra::DMatrix;
use rand::thread_rng;
use rand_distr::{Distribution, Normal};
use std::{collections::HashMap, usize};

struct VolatilitySimulation {
    num_days: usize,
    daily_volatility: f64,
}

impl VolatilitySimulation {
    fn new(num_days: usize, daily_volatility: f64) -> Self {
        VolatilitySimulation {
            num_days,
            daily_volatility,
        }
    }

    fn wiener_process(&self) -> StatisticalProcessSimulation {
        let nsteps = self.num_days * 24;
        let sigma = self.daily_volatility / 24.0;

        let mut rng = thread_rng();
        let normal = Normal::new(0.0, sigma).unwrap();

        let mut brownian = DMatrix::from_fn(1000, nsteps, |_, _| normal.sample(&mut rng));
        cumsum(&mut brownian, Some(0));

        let res: Vec<f64> = brownian.column(nsteps - 1).map(|e| e).data.into();

        StatisticalProcessSimulation(res)
    }

    // NOTE: design is to allow for consistent creation of
    // alternate processes, e.g.
    // fn poisson_process(&self) -> StatisticalProcessSimulation {
    //     todo!()
    // }
}

struct StatisticalProcessSimulation(Vec<f64>);

impl StatisticalProcessSimulation {
    fn quantiles(&self, qlist: Option<&[f64]>) -> HashMap<String, f64> {
        let n: f64 = self.0.len() as f64;
        let mut sorted_observations = self.0.clone();
        sorted_observations.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mut res: HashMap<String, f64> = HashMap::new();
        let qlist = qlist.unwrap_or(&[0.025, 0.250, 0.500, 0.750, 0.975]);
        for e in qlist.iter() {
            let idx: usize = (n * e) as usize;
            let key: String = (e * 100.0).to_string();
            let val: f64 = sorted_observations[idx];
            res.insert(key, val);
        }

        res
    }
}

fn cumsum(arr: &mut DMatrix<f64>, axis: Option<i32>) {
    // Only implemented for 2D arrays as nalgebra is stil very WIP
    // axis == 0 --> row-wise (default)
    // axis != 0 --> column-wise
    let axis = axis.unwrap_or(0);

    if axis == 0 {
        for col in 0..arr.ncols() {
            if col > 0 {
                let rolling_sum = arr.column(col) + arr.column(col - 1);
                for row in 0..arr.nrows() {
                    *arr.index_mut((row, col)) = rolling_sum[row];
                }
            }
        }
    } else {
        for row in 0..arr.nrows() {
            if row > 0 {
                let rolling_sum = arr.row(row) + arr.row(row - 1);
                for col in 0..arr.ncols() {
                    *arr.index_mut((row, col)) = rolling_sum[col];
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct CreateLenderSuggestions {
    risk_appetite: RiskAppetite,
    bet_low: f64,
    bet_high: f64,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum RiskAppetite {
    Low,
    Moderate,
    High,
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct LenderSuggestionParameters {
    mu: f64,
    sigma: f64,
    lvr: f64,
    max_interest_rate: f64,
}

impl LenderSuggestionParameters {
    fn new(mu: f64, sigma: f64, lvr: f64, max_interest_rate: f64) -> Self {
        LenderSuggestionParameters {
            mu,
            sigma,
            lvr,
            max_interest_rate,
        }
    }
}

impl CreateLenderSuggestions {
    fn new(risk_appetite: RiskAppetite, mut bet_low: f64, mut bet_high: f64) -> Self {
        if bet_high < bet_low {
            std::mem::swap(&mut bet_low, &mut bet_high)
        }

        if bet_low <= -1.0 {
            bet_low = -0.999999;
        }

        if bet_low <= -1.0 {
            bet_high = -0.999999;
        }

        CreateLenderSuggestions {
            risk_appetite,
            bet_low,
            bet_high,
        }
    }

    fn suggest_parameters(&self, interest_rate: Option<f64>) -> LenderSuggestionParameters {
        let mut rval: f64 = 1.0;

        if self.risk_appetite == RiskAppetite::Low {
            rval *= 3.0;
        } else if self.risk_appetite == RiskAppetite::Moderate {
            rval *= 2.0;
        }

        let mu: f64 = 0.5 * (self.bet_high - self.bet_low) + self.bet_low;
        let sigma: f64 = (mu - self.bet_low) / rval;
        let irate = interest_rate.unwrap_or(mu + 3.0 * sigma);

        let test_uvals = [mu - 3.0 * sigma, -0.999999];
        let min_u = test_uvals.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        let test_lvr = [(1.0 + min_u) / (1.0 + irate), 1.0];
        let lvr = test_lvr.iter().copied().fold(f64::INFINITY, f64::min);

        LenderSuggestionParameters::new(mu, sigma, lvr, irate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_plausible_wiener() {
        let ndays: usize = 30;
        let volatility: f64 = 0.046;
        let simulation = VolatilitySimulation::new(ndays, volatility);
        let quants = simulation.wiener_process().quantiles(None);

        // NOTE: the better way to do this is set a seed value and get
        // deterministic values, but I don't know how to actually do
        // this at this point in time.
        let q_testval_025 = *quants.get("2.5").unwrap();
        let q_testval_250 = *quants.get("25").unwrap();
        let q_testval_500 = *quants.get("50").unwrap();

        assert!(q_testval_025 >= -0.11 && q_testval_025 <= -0.09);
        assert!(q_testval_250 >= -0.05 && q_testval_250 <= -0.03);
        assert!(q_testval_500 >= -0.005 && q_testval_500 <= 0.005);
    }

    #[test]
    fn check_suggestions() {
        for risk in [
            RiskAppetite::High,
            RiskAppetite::Moderate,
            RiskAppetite::Low,
        ]
        .iter()
        {
            let suggestions = CreateLenderSuggestions::new(*risk, 0.5, -10.0);
            let params = suggestions.suggest_parameters(None);
            assert!(suggestions.risk_appetite == *risk);
            assert!(suggestions.bet_low == -0.999999);
            assert!(suggestions.bet_high == 0.5);
            assert!(params.lvr <= 10e-6);
        }
    }
}
