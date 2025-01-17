use std::env;

use std::path::PathBuf;

use anyhow::Result;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, signature::Keypair, signer::Signer,
    transaction::Transaction,
};

use crate::{
    commands::instructions::create_mint_solmap_ix, constants::PRIORITY_FEE_RATE, setup::CliConfig,
};

pub struct MintArgs {
    pub keypair_path: Option<PathBuf>,
    pub rpc_url: Option<String>,
    pub solmap_number: u64,
    pub boost: bool,
}

pub fn mint(args: MintArgs) -> Result<()> {
    println!("Minting solmap number {}", args.solmap_number);
    let config = CliConfig::new(args.keypair_path, args.rpc_url)?;

    let mint = Keypair::new();

    let compute_budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(400_000);
    let mut instructions = vec![compute_budget_ix];

    if args.boost {
        let priority_fee_rate = env::var("PRIORITY_FEE_RATE")
            .unwrap_or_else(|_| PRIORITY_FEE_RATE.to_string())
            .parse::<u64>()
            .expect("Invalid PRIORITY_FEE_RATE value");
    
        instructions.push(ComputeBudgetInstruction::set_compute_unit_price(
            priority_fee_rate,
        ));
        let priority_fee_rate_in_sol = priority_fee_rate as f64 / 1_000_000_000.0;
        println!("Priority fee {} SOL", priority_fee_rate_in_sol);
    }
    
    instructions.push(create_mint_solmap_ix(
        config.keypair.pubkey(),
        mint.pubkey(),
        args.solmap_number,
    ));

    let blockhash = config.client.get_latest_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&config.keypair.pubkey()),
        &[&config.keypair, &mint],
        blockhash,
    );
    let sig = config
        .client
        .send_and_confirm_transaction_with_spinner(&tx)?;

    println!(
        "Minted solmap number {} with signature {}",
        args.solmap_number, sig
    );

    Ok(())
}
