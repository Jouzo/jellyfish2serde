export interface AddressInfo {
  embedded: {
    address: string
    scriptPubKey: string
    isscript: boolean
    iswitness: boolean
    witness_version: number
    witness_program: string
    script: ScriptType
    hex: string,
    sigsrequired: number
    pubkey: string
    pubkeys: string[]
  }
}
