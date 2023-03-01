use {
  super::*,
  crate::wallet::Wallet,
};

#[derive(Serialize)]
struct Output {
  fees: u64,
}

#[derive(Debug, Parser)]
pub(crate) struct Estimate {
  #[clap(long, help = "Inscribe <SATPOINT>")]
  pub(crate) satpoint: Option<SatPoint>,
  #[clap(
    long,
    default_value = "1.0",
    help = "Use fee rate of <FEE_RATE> sats/vB"
  )]
  pub(crate) fee_rate: FeeRate,
  #[clap(
    long,
    help = "Use <COMMIT_FEE_RATE> sats/vbyte for commit transaction.\nDefaults to <FEE_RATE> if unset."
  )]
  pub(crate) commit_fee_rate: Option<FeeRate>,
  #[clap(help = "Inscribe sat with contents of <FILE>")]
  pub(crate) file: PathBuf,
  #[clap(
    long,
    help = "Do not check that transactions are equal to or below the MAX_STANDARD_TX_WEIGHT of 400,000 weight units. Transactions over this limit are currently nonstandard and will not be relayed by bitcoind in its default configuration. Do not use this flag unless you understand the implications."
  )]
  pub(crate) no_limit: bool,
  #[clap(long, help = "Ignore the index in order to optimise.")]
  pub(crate) no_index: bool,
}

impl Estimate {
  pub(crate) fn run(self, options: Options) -> Result {
    let inscription = Inscription::from_file(options.chain(), &self.file)?;

    let (mut utxos, inscriptions) = if self.no_index {
      let mut utxos = BTreeMap::new();
      utxos.insert(OutPoint::null(), Amount::MAX_MONEY);
      (utxos, BTreeMap::new())
    } else {
      let index = Index::open(&options)?;
      index.update()?;
      (index.get_unspent_outputs(Wallet::load(&options)?)?, index.get_inscriptions(None)?)
    };

    let commit_tx_change = ["32iVBEu4dxkUQk9dJbZUiBiQdmypcEyJRf".parse().unwrap(), "1QJVDzdqb1VpbDK7uDeyVXy9mR27CJiyhY".parse().unwrap()];

    let reveal_tx_destination = "32iVBEu4dxkUQk9dJbZUiBiQdmypcEyJRf".parse().unwrap();

    let (unsigned_commit_tx, reveal_tx, _) =
      inscribe::Inscribe::create_inscription_transactions(
        self.satpoint,
        inscription,
        inscriptions,
        options.chain().network(),
        utxos.clone(),
        commit_tx_change,
        reveal_tx_destination,
        self.commit_fee_rate.unwrap_or(self.fee_rate),
        self.fee_rate,
        self.no_limit,
      )?;

    utxos.insert(
      reveal_tx.input[0].previous_output,
      Amount::from_sat(unsigned_commit_tx.output[0].value),
    );

    let fees =
      inscribe::Inscribe::calculate_fee(&unsigned_commit_tx, &utxos) + inscribe::Inscribe::calculate_fee(&reveal_tx, &utxos);

    print_json(Output {
      fees,
    })?;

    Ok(())
  }
}
