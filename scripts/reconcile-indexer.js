#!/usr/bin/env node
/**
 * reconcile-indexer.js
 *
 * Compares on-chain contract state against the local PostgreSQL indexer tables
 * and reports discrepancies.  Designed to be run as a cron job or manual
 * diagnostic tool.
 *
 * Usage:
 *   node scripts/reconcile-indexer.js [--contract <address>]
 *
 * Environment:
 *   DATABASE_URL  - PostgreSQL connection string
 *   RPC_URL       - Soroban RPC endpoint
 */

const { Client } = require('pg');
const { SorobanRpc } = require('@stellar/stellar-sdk');

const RPC_URL = process.env.RPC_URL || 'https://soroban-testnet.stellar.org';
const DATABASE_URL = process.env.DATABASE_URL || 'postgresql://localhost/scoutchain';

const client = new Client({ connectionString: DATABASE_URL });

async function connectDb() {
  await client.connect();
}

async function closeDb() {
  await client.end();
}

async function fetchPlayersFromChain() {
  // Placeholder: in production this would query the registration contract's
  // filter_players or iterate stored player IDs via RPC.
  return [];
}

async function fetchScoutsFromChain() {
  // Placeholder: in production this would query the registration contract's
  // scout registry via RPC.
  return [];
}

async function reconcilePlayers() {
  const chainPlayers = await fetchPlayersFromChain();
  const { rows } = await client.query('SELECT player_id, wallet, deactivated FROM players');

  const dbMap = new Map(rows.map((r) => [r.player_id, r]));
  const chainMap = new Map(chainPlayers.map((p) => [p.id, p]));

  const missingInDb = chainPlayers.filter((p) => !dbMap.has(p.id));
  const missingOnChain = rows.filter((r) => !chainMap.has(r.player_id));

  for (const p of missingInDb) {
    console.warn(`[reconcile] player ${p.id} exists on-chain but missing in DB`);
  }

  for (const r of missingOnChain) {
    console.warn(`[reconcile] player ${r.player_id} exists in DB but missing on-chain`);
  }

  // Check deactivated flag
  for (const r of rows) {
    const chain = chainMap.get(r.player_id);
    if (!chain) continue;
    const chainDeactivated = chain.deactivated === true;
    if (r.deactivated !== chainDeactivated) {
      console.warn(
        `[reconcile] player ${r.player_id} deactivated mismatch: db=${r.deactivated} chain=${chainDeactivated}`
      );
    }
  }

  return { missingInDb, missingOnChain };
}

async function reconcileScouts() {
  const chainScouts = await fetchScoutsFromChain();
  const { rows } = await client.query('SELECT scout_id, wallet, verified FROM scouts');

  const dbMap = new Map(rows.map((r) => [r.scout_id, r]));
  const chainMap = new Map(chainScouts.map((s) => [s.id, s]));

  const missingInDb = chainScouts.filter((s) => !dbMap.has(s.id));
  const missingOnChain = rows.filter((r) => !chainMap.has(r.scout_id));

  for (const s of missingInDb) {
    console.warn(`[reconcile] scout ${s.id} exists on-chain but missing in DB`);
  }

  for (const r of missingOnChain) {
    console.warn(`[reconcile] scout ${r.scout_id} exists in DB but missing on-chain`);
  }

  // Check verified flag
  for (const r of rows) {
    const chain = chainMap.get(r.scout_id);
    if (!chain) continue;
    const chainVerified = chain.verified === true;
    if (r.verified !== chainVerified) {
      console.warn(
        `[reconcile] scout ${r.scout_id} verified mismatch: db=${r.verified} chain=${chainVerified}`
      );
    }
  }

  return { missingInDb, missingOnChain };
}

async function main() {
  console.log('Starting indexer reconciliation...');
  await connectDb();

  try {
    const playerReport = await reconcilePlayers();
    const scoutReport = await reconcileScouts();

    const hasIssues =
      playerReport.missingInDb.length > 0 ||
      playerReport.missingOnChain.length > 0 ||
      scoutReport.missingInDb.length > 0 ||
      scoutReport.missingOnChain.length > 0;

    if (hasIssues) {
      console.error('Reconciliation found discrepancies');
      process.exitCode = 1;
    } else {
      console.log('Reconciliation passed: no discrepancies found');
    }
  } finally {
    await closeDb();
  }
}

main().catch((err) => {
  console.error('Reconciliation failed:', err);
  process.exit(1);
});
