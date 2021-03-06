//!
//! The Zinc tester arguments.
//!

use structopt::StructOpt;

///
/// The Zinc tester arguments.
///
#[derive(Debug, StructOpt)]
#[structopt(
    name = zinc_const::app_name::TESTER,
    about = "The integration test runner for the Zinc framework"
)]
pub struct Arguments {
    /// Prints more logs, if passed several times.
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbosity: usize,
    /// Runs the full testing with trusted setup and proof verification.
    #[structopt(short = "p", long = "proof-check")]
    pub proof_check: bool,
    /// Runs only tests whose name contains the specified string.
    #[structopt(short = "f", long = "filter")]
    pub filter: Option<String>,
}

impl Arguments {
    ///
    /// A shortcut constructor.
    ///
    pub fn new() -> Self {
        Self::from_args()
    }
}
