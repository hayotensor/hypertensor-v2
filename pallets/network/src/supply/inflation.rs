// Copyright (C) Hypertensor.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// Enables accounts to delegate stake to subnets for a portion of emissions

use libm::pow;

pub struct Inflation {
  initial: f64,       // Initial inflation rate (8%)
  terminal: f64,    // Annual decay rate (15%)
  taper: f64,     // Minimum inflation rate (1.5%)
  epochs_per_year: u64, // Number of epochs per year
}

const DEFAULT_INITIAL: f64 = 0.08;
const DEFAULT_TERMINAL: f64 = 0.015;
const DEFAULT_TAPER: f64 = 0.15;
const DEFAULT_EPOCHS_PER_YEAR: u64 = 52_594;

impl Default for Inflation {
  fn default() -> Self {
    Self {
      initial: DEFAULT_INITIAL,
      terminal: DEFAULT_TERMINAL,
      taper: DEFAULT_TAPER,
      epochs_per_year: DEFAULT_EPOCHS_PER_YEAR,
    }
  }
}

impl Inflation {
  pub fn epoch(&self, epoch: u64) -> u128 {
    let years_elapsed = epoch as f64 / self.epochs_per_year as f64;
    // let rate = self.initial * (1.0 - self.terminal).powf(years_elapsed);
    let rate = self.initial * pow(1.0 - self.terminal, years_elapsed);

    // Ensure inflation does not go below the minimum taper rate
    let final_rate = rate.max(self.taper);

    // Convert to u128 with 1e18 scaling
    (final_rate * 1e+18) as u128
  }

  /// inflation rate at year
  pub fn total(&self, year: f64) -> f64 {
    let tapered = self.initial * pow(1.0 - self.taper, year);

    if tapered > self.terminal {
      tapered
    } else {
      self.terminal
    }
  }
}
