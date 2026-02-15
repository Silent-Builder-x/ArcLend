# ArcLend: MPC-Powered Confidential Lending & Liquidation Engine

## 🏦 Overview

**ArcLend** is a privacy-preserving lending protocol built on **Arcium** and **Solana**.

In traditional DeFi lending (e.g., Aave, Compound), a user's health factor, collateral value, and debt are entirely transparent. This transparency allows predatory actors to calculate precise "liquidation points" and manipulate market prices to trigger liquidations. **ArcLend** solves this by performing risk assessment and liquidation checks entirely on **Secret-Shared Data** within Arcium's Multi-Party Execution (MXE) environment.

## 🚀 Live Deployment Status (Devnet v0.8.3)

The protocol is fully operational and verified on the Arcium Devnet.

### 🖥️ Interactive Demo

[Launch ArcLend Terminal](https://silent-builder-x.github.io/ArcLend/)

## 🧠 Core Innovation: The "Hidden" Health Factor

ArcLend utilizes Arcis MPC circuits to implement a **Confidential Risk Engine**:

- **Shielded Collateralization:** Users deposit and borrow against secret shares. The exact LTV (Loan-to-Value) ratio is hidden from the public ledger.
- **Automated Oblivious Liquidation:** The circuit computes `(Collateral * LTV_Threshold) / Debt` on encrypted inputs. It only outputs a boolean flag (0 or 1) to indicate if a position is liquidatable, without revealing the underlying financial data.
- **MEV Resistance:** Prevents "Liquidation Sniping" by keeping the proximity to the liquidation threshold invisible to bot operators.

## 🛠 Build & Implementation

```
# Compile the Arcis circuit and Solana program
arcium build

# Deploy to Cluster 456
arcium deploy --cluster-offset 456 --recovery-set-size 4 --keypair-path ~/.config/solana/id.json -u d

```

## 📄 Technical Specification

- **Engine:** `check_liquidation` (Arcis-MPC Circuit)
- **Settlement:** Verified MXE Callback via `CheckLiquidationCallback`
- **Security:** Supported by Arcium's Multi-Party Execution and Recovery Set.