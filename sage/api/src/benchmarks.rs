pub trait SageBenchmarkHelper<
	AssetId,
	Asset,
	TransitionId,
	TradeFilter,
	TransferFilter,
	PaymentKind,
>
{
	fn create_asset(seed: u32) -> (AssetId, Asset);

	fn create_bench_transition() -> (TransitionId, sp_std::vec::Vec<AssetId>);

	fn create_trade_filter_for(asset: &Asset) -> TradeFilter;

	fn create_transfer_filter_for(asset: &Asset) -> TransferFilter;

	fn create_payment_kind() -> PaymentKind;
}
