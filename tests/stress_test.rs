// Copyright (c) 2019, Arm Limited, All Rights Reserved
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may
// not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//          http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#[cfg(test)]
mod tests {
    use parsec_client_test::{StressTestClient, StressTestConfig};
    use std::time::Duration;

    #[test]
    fn stress_test() {
        let config = StressTestConfig {
            no_threads: num_cpus::get(),
            req_per_thread: 250,
            req_interval: Some(Duration::from_millis(10)),
            req_interval_deviation_millis: Some(4),
            check_interval: Some(Duration::from_millis(500)),
        };

        StressTestClient::execute(config);
    }
}
