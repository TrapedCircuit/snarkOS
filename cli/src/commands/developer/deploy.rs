// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the snarkOS library.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at:
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::{CurrentNetwork, Developer};

use snarkvm::{
    prelude::{ConsensusStore, Plaintext, PrivateKey, ProgramID, Query, Record, VM},
    synthesizer::store::helpers::memory::ConsensusMemory,
};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::str::FromStr;

/// Deploys an Aleo program.
#[derive(Debug, Parser)]
pub struct Deploy {
    /// The name of the program to deploy.
    #[clap(parse(try_from_str))]
    program_id: ProgramID<CurrentNetwork>,
    /// A path to a directory containing a manifest file. Defaults to the current working directory.
    #[clap(long)]
    path: Option<String>,
    /// The private key used to generate the deployment.
    #[clap(short, long)]
    private_key: String,
    /// The endpoint to query node state from.
    #[clap(short, long)]
    query: String,
    /// The deployment fee in microcredits.
    #[clap(short, long)]
    fee: u64,
    /// The record to spend the fee from.
    #[clap(short, long)]
    record: String,
    /// Display the generated transaction.
    #[clap(short, long, conflicts_with = "broadcast")]
    display: bool,
    /// The endpoint used to broadcast the generated transaction.
    #[clap(short, long, conflicts_with = "display")]
    broadcast: Option<String>,
    /// Store generated deployment transaction to a local file.
    #[clap(long)]
    store: Option<String>,
}

impl Deploy {
    /// Deploys an Aleo program.
    pub fn parse(self) -> Result<String> {
        // Specify the query
        let query = Query::from(self.query);

        // Retrieve the private key.
        let private_key = PrivateKey::from_str(&self.private_key)?;

        // Fetch the program from the directory.
        let program = Developer::parse_program(self.program_id, self.path)?;

        println!("📦 Creating deployment transaction for '{}'...\n", &self.program_id.to_string().bold());

        // Generate the deployment transaction.
        let deployment = {
            // Initialize an RNG.
            let rng = &mut rand::thread_rng();

            // Initialize the VM.
            let store = ConsensusStore::<CurrentNetwork, ConsensusMemory<CurrentNetwork>>::open(None)?;
            let vm = VM::from(store)?;

            // Prepare the fees.
            let fee = (Record::<CurrentNetwork, Plaintext<CurrentNetwork>>::from_str(&self.record)?, self.fee);

            // Create a new transaction.
            vm.deploy(&private_key, &program, fee, Some(query), rng)?
        };
        println!("✅ Created deployment transaction for '{}'", self.program_id.to_string().bold());

        // Determine if the transaction should be broadcast, stored, or displayed to user.
        Developer::handle_transaction(self.broadcast, self.display, self.store, deployment, self.program_id.to_string())
    }
}
