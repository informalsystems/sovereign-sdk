#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    demo_stf::cli::run_wallet::<
        <sov_celestia_adapter::CelestiaService as sov_rollup_interface::services::da::DaService>::Spec,
    >()
    .await
}
