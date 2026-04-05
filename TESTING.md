# Alepic Testing Methodology

## Overview

This document outlines the comprehensive testing strategy for the Alepic project, with a critical focus on smart contract security. Given that the smart contract manages a treasury and facilitates financial transactions, rigorous testing is paramount to prevent bugs, hacks, and exploits.

**Core Security Principle:** The Treasury must be impervious to unauthorized external spending. All fund movements must be strictly governed by the protocol rules defined in the code.

---

## 1. Smart Contract Security Strategy

The smart contract (`contracts/AlepicMain.ralph`) is the most critical component. Its testing strategy is multi-layered:

### 1.1. Threat Model & Attack Vectors

We explicitly test against the following attack vectors:

| Attack Vector | Description | Mitigation Strategy |
| :--- | :--- | :--- |
| **Reentrancy** | Attacker calls back into the contract during a state update to drain funds. | **Checks-Effects-Interactions** pattern. State variables (ownership, balances) are updated *before* any ALPH transfer occurs. |
| **Unauthorized Treasury Access** | External actor attempts to withdraw from the Treasury directly. | **No public withdrawal functions.** Treasury funds can only move via specific game mechanics (Alepe reward) or fee distribution logic, all strictly bounded by math. |
| **Integer Overflow/Underflow** | Math errors leading to incorrect balance calculations. | Ralph uses safe math by default, but we explicitly test boundary conditions (max u256, zero balances). |
| **Front-Running** | Attacker sees a profitable transaction and inserts their own before it. | Logic does not rely on transaction ordering for security. Auctions use explicit bid amounts, not "first come first served" for the same price. |
| **Logic Exploits** | Finding edge cases in fee calculation or auction logic to steal funds. | Exhaustive property-based testing of all financial formulas. |
| **Access Control Bypass** | Calling admin-only functions without permission. | Strict `#[using_preapproved!]` and signer checks on all privileged functions. |

### 1.2. Smart Contract Test Suite Structure

Since Ralph contracts compile to bytecode for the Alephium VM, testing is performed using the **Alephium TypeScript SDK** and **Jest**.

#### Directory Structure
```text
contracts/
  AlepicMain.ralph       # Source code
test/
  contract/
    utils.ts             # Test helpers (deploy, sign, assert)
    Alepic.test.ts       # Main test suite
    AlepeGame.test.ts    # Game mechanic specific tests
    Treasury.test.ts     # Financial security tests
    Marketplace.test.ts  # Buying/Selling/Auction tests
```

#### Key Test Categories

##### A. Treasury Protection Tests (`Treasury.test.ts`)
These are the most critical tests. They must prove that the Treasury balance can never decrease unless explicitly allowed by protocol rules.

1.  **Direct Withdrawal Attempt**: Verify that calling a hypothetical `withdraw` function (if it existed) reverts.
2.  **Fee Distribution Integrity**:
    *   Deposit exact amount for a chunk sale.
    *   Assert Treasury receives exactly 95% (initial) or 4% (secondary).
    *   Assert Referrer/Seller receive exact remainders.
    *   **Invariant**: `Sum(Outputs) == Input`. No dust left behind, no extra created.
3.  **Alepe Reward Mechanism**:
    *   Simulate block height reaching a multiple of 100,000.
    *   Verify Treasury decreases by exactly 1%.
    *   Verify the specific chunk owner receives the exact amount.
    *   **Crucial**: Ensure this cannot be triggered manually or prematurely.
4.  **Reentrancy Guard**:
    *   Deploy a malicious attacker contract.
    *   Attempt to buy a chunk where the `onPurchase` hook tries to call back into Alepic.
    *   Assert transaction fails or state remains unchanged.

##### B. Marketplace Logic Tests (`Marketplace.test.ts`)
1.  **Ownership Transfer**: Verify ownership only changes upon successful payment.
2.  **Auction Logic**:
    *   Test bidding higher than current bid.
    *   Test refunding previous bidder.
    *   Test settling auction: Ensure highest bidder gets chunk, seller gets funds (minus fees).
3.  **Referrer Logic**: Test with null referrer, valid referrer, and self-referral.

##### C. Alepe Game Logic Tests (`AlepeGame.test.ts`)
1.  **Jump Timing**: Verify jumps occur *only* at $blockHeight \% 100,000 == 0$.
2.  **Position Calculation**: Verify new coordinates are within bounds and wrap correctly.
3.  **Chunk Identification**: Verify the contract correctly identifies the 4 chunks (2x2) Alepe occupies after a jump.

### 1.3. Example Test Snippet (TypeScript/Jest)

```typescript
// test/contract/Treasury.test.ts
import { deployContract, createAccount } from './utils';

describe('Treasury Security', () => {
  let alepic: ContractInstance;
  let owner: Account;
  let attacker: Account;

  beforeEach(async () => {
    owner = await createAccount();
    attacker = await createAccount();
    alepic = await deployContract(owner, { initialSupply: 1000n });
  });

  it('REVERTS if non-protocol function attempts to withdraw from Treasury', async () => {
    const initialBalance = await alepic.getTreasuryBalance();
    
    // Attempt to call a protected function or simulate an exploit
    // Since there is no public withdraw, we try to manipulate fee inputs
    // to see if we can extract more than we put in.
    
    try {
        // Hypothetical exploit attempt
        await alepic.methods.exploitTreasury().transact({ from: attacker });
        fail('Transaction should have reverted');
    } catch (e) {
        expect(e.message).toContain('revert');
    }

    const finalBalance = await alepic.getTreasuryBalance();
    expect(finalBalance).toBe(initialBalance);
  });

  it('Distributes fees correctly and preserves invariant', async () => {
    const price = 1000n;
    const referrer = await createAccount();
    
    const treasuryBefore = await alepic.getTreasuryBalance();
    const referrerBefore = await getBalance(referrer.address);

    // Execute Sale
    await alepic.methods.buyChunk(chunkId, referrer.address).transact({ 
        from: owner, 
        amount: price 
    });

    const treasuryAfter = await alepic.getTreasuryBalance();
    const referrerAfter = await getBalance(referrer.address);

    const treasuryFee = price * 95n / 100n;
    const referrerFee = price * 5n / 100n;

    expect(treasuryAfter - treasuryBefore).toBe(treasuryFee);
    expect(referrerAfter - referrerBefore).toBe(referrerFee);
    
    // Invariant Check: Total money in system is conserved (minus gas)
    // This is a conceptual check; actual implementation tracks internal balances
  });
});
```

---

## 2. Rust Backend & Desktop App Testing

The Rust codebase (`src/`) handles the UI, local canvas state, and interaction with the blockchain.

### 2.1. Unit Testing Strategy

Located in `tests/` directory.

| Module | Focus Area | Key Tests |
| :--- | :--- | :--- |
| `canvas_tests.rs` | Data Integrity | Pixel set/get, chunk dirty tracking, coordinate mapping (screen <-> world), serialization/deserialization. |
| `alepe_tests.rs` | Game Logic | Jump intervals, position wrapping, occupied chunk calculation, collision detection. |
| `fees_tests.rs` | Financial Math | Exact fee calculations (initial/secondary), rounding errors, overflow protection. |
| `content_filter_tests.rs` | Moderation | Pattern detection (solid colors, flashing), blocked list matching, neural network mock responses. |

### 2.2. Integration Testing

Tests the interaction between the UI and the Blockchain module.

*   **Mock Blockchain**: A trait implementation `MockBlockchain` that simulates network delays, successful transactions, and failures without hitting the real Alephium network.
*   **Scenario**: User clicks "Buy" -> UI constructs transaction -> Mock Blockchain validates signature -> Updates local state.

### 2.3. Visual Regression Testing

Since Alepic is a visual application, we employ pixel-perfect testing for the UI.

*   **Tool**: `cargo-insta` or custom screenshot comparison.
*   **Method**: Render specific states (e.g., "Alepe jumping", "Auction modal open") and compare against stored baseline images.
*   **Goal**: Detect unintended UI shifts, color palette errors, or rendering artifacts.

---

## 3. Security Audit Checklist

Before any mainnet deployment, the following checklist must be completed:

### 3.1. Code Review
- [ ] **Manual Review**: Every line of `AlepicMain.ralph` reviewed by at least two senior developers.
- [ ] **Logic Walkthrough**: Step-by-step execution of fund flows on a whiteboard.
- [ ] **Dependency Check**: All imported libraries (Rust crates, Ralph modules) scanned for vulnerabilities (`cargo audit`).

### 3.2. Automated Analysis
- [ ] **Static Analysis**: Run Ralph linter and custom scripts to detect common patterns (e.g., unprotected state changes).
- [ ] **Fuzzing**: Use fuzzers on the Rust fee calculation logic to find edge cases that cause panics or incorrect math.

### 3.3. Formal Verification (Optional but Recommended)
- For critical functions (Treasury withdrawal logic), consider writing formal specifications to mathematically prove correctness.

### 3.4. External Audit
- **Mandatory**: Hire a third-party smart contract auditing firm specializing in Alephium/Ralph.
- **Scope**: Full review of Treasury management, access control, and economic incentives.

---

## 4. Continuous Integration (CI) Pipeline

All tests run automatically on every Pull Request via GitHub Actions.

### Pipeline Stages

1.  **Lint & Format**: `cargo fmt --check`, `cargo clippy`, Ralph linter.
2.  **Unit Tests**: `cargo test --all` (Must pass 100%).
3.  **Contract Tests**: `npm run test:contract` (Runs Jest suite against local Alephium node).
4.  **Security Scan**: `cargo audit`, dependency vulnerability check.
5.  **Build**: `cargo build --release` (Ensures code compiles in release mode).

### CI Configuration Example (`.github/workflows/test.yml`)

```yaml
name: Test & Security

on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      - name: Run Unit Tests
        run: cargo test --verbose
      - name: Security Audit
        run: cargo install cargo-audit && cargo audit

  contract-tests:
    runs-on: ubuntu-latest
    services:
      alephium-node:
        image: alephium/full-node:latest
        ports:
          - 12973:12973
    steps:
      - uses: actions/checkout@v3
      - name: Setup Node
        uses: actions/setup-node@v3
      - name: Install Deps
        run: npm ci
      - name: Run Contract Tests
        run: npm run test:contract
```

---

## 5. Test Execution Guide

### Running Rust Tests
```bash
# Run all tests
cargo test

# Run specific module
cargo test --test fees_tests

# Run with output logs
cargo test -- --nocapture
```

### Running Smart Contract Tests
```bash
# Ensure local Alephium node is running or use mocked provider
npm install
npm run test:contract

# Run with coverage (if configured)
npm run test:coverage
```

### Adding New Tests
1.  **Rust**: Add function to existing `tests/*_tests.rs` file. Use `#[test]` attribute.
2.  **Contract**: Add `it()` block to relevant `test/contract/*.test.ts` file.
3.  **Rule**: Any new feature *must* include corresponding tests before merging.

---

## 6. Incident Response & Bug Bounty

If a vulnerability is discovered:

1.  **Pause Mechanism**: If the contract includes an emergency pause (owned by multisig), activate it immediately.
2.  **Disclosure**: Report privately to the core team. Do not disclose publicly until fixed.
3.  **Fix & Deploy**: Patch the contract, re-run full test suite, audit, and deploy new version.
4.  **Migration**: If state is compromised, plan a migration strategy for users.

**Bug Bounty Program**:
We encourage ethical hackers to report issues. Rewards are offered for critical vulnerabilities, especially those related to Treasury theft.

---

## Conclusion

Testing in Alepic is not just about correctness; it is about **financial security**. The separation of concerns between the Rust UI (which can be buggy without losing funds) and the Ralph Smart Contract (where bugs are catastrophic) dictates our strategy. The Smart Contract tests are exhaustive, focusing on invariants and attack vectors, while the Rust tests ensure a smooth user experience and correct local state management.
