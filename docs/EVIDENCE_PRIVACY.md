# Encrypted Evidence and Access Grants

## Security model

The CID stored in a milestone or trial offer remains a public, immutable
content reference. It proves which payload was approved at a given time, but it
does not provide confidentiality. Privacy comes from making the referenced
payload client-side encrypted before it is uploaded to IPFS or Arweave.

| Property | Mechanism |
|---|---|
| Tamper-proof reference | On-chain `evidence_hash` / `details_hash` CID and approval event |
| Confidential media | Encrypted payload stored at that CID |
| Viewer authorization | `EvidenceAccessGrant` emitted after successful `pay_to_contact` |

## Key distribution

1. The player client creates a random content-encryption key, encrypts the
   evidence locally, and uploads only ciphertext.
2. The player retains its own key material off-chain. This contract has no
   player-wallet registry, so the player's initial key access is not modeled as
   an on-chain grant.
3. A successful `pay_to_contact` atomically writes an `EvidenceAccessGrant`
   for `(player_id, scout)` and emits `evidence_access_granted`.
4. The frontend/backend watches that event or reads the grant, verifies it, and
   delivers a viewer-specific wrapped key. The viewer decrypts locally.

The smart contracts never receive plaintext media, raw encryption keys, or
wrapped keys. The frontend/backend repository owns encryption, wallet-key
handling, key wrapping, retrieval, and decryption.

## Contract scope

This repository adds the grant storage, query API, and event to the existing
subscription and contact-payment authorization flow. It does not make a public
CID private by itself: uploads must be encrypted before use for the access
grant to have confidentiality value.

## Migration

Existing CIDs may already point to unencrypted content and cannot be made
private by adding an on-chain flag. Treat them as legacy public evidence. To
migrate, encrypt the original media, upload the ciphertext under a new CID,
and publish the replacement through an application-level evidence-versioning
workflow; do not overwrite historical approval records.
