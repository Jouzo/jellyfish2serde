export interface VaultLiquidation extends Vault {
  liquidationHeight: number
  liquidationPenalty: number
  batchCount: number
  batches: VaultLiquidationBatch[]
}
