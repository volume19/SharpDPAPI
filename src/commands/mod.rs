// Commands module

use std::collections::HashMap;
use anyhow::Result;

pub trait Command {
    fn execute(&self, arguments: &HashMap<String, String>) -> Result<()>;
}

// Command modules will be added here as they are implemented
// pub mod backupkey;
// pub mod blob;
// pub mod credentials;
// pub mod keepass;
// pub mod machinecredentials;
// pub mod machinemasterkeys;
// pub mod machinetriage;
// pub mod machinevaults;
// pub mod masterkeys;
// pub mod ps;
// pub mod rdg;
// pub mod triage;
// pub mod vaults;
// pub mod certificates;
// pub mod search;
// pub mod sccm;
