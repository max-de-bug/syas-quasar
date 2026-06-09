import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import {
  airdrop,
  findPda,
} from "./helpers";
import { TestProvider } from "./helpers/provider";
import { PROGRAM_IDS } from "./helpers/constants";
import {
  RENT_SYSVAR,
  buildRegistryInitialize,
  buildProposeAdapter,
  buildApproveAdapter,
  buildRevokeAdapter,
  buildTransferGovernance,
  deserializeRegistryState,
  deserializeAdapterEntry,
} from "./helpers/quasar-client";

describe("adapter-registry", () => {
  const provider = TestProvider.env();
  const authority = provider.wallet;

  let registryStatePda: PublicKey;

  before(async () => {
    [registryStatePda] = findPda(
      [Buffer.from("registry_state")],
      PROGRAM_IDS.adapterRegistry
    );
  });

  it("initializes the registry", async () => {
    try {
      const ix = buildRegistryInitialize(
        authority.publicKey,
        registryStatePda,
      );
      await provider.sendIx(ix);
    } catch (e: unknown) {
      const msg = String(e);
      if (!msg.includes("already in use") && !msg.includes("0x0") && !msg.includes("requires an uninitialized")) {
        throw e;
      }
    }

    const info = await provider.connection.getAccountInfo(registryStatePda);
    expect(info).to.not.be.null;
    const state = deserializeRegistryState(info!.data);
    expect(state.authority.toString()).to.equal(authority.publicKey.toString());
    expect(state.totalProposed).to.be.at.least(0);
    expect(state.totalApproved).to.be.at.least(0);
  });

  it("proposes an adapter", async () => {
    const adapterProgram = Keypair.generate();
    const underlyingMint = Keypair.generate();

    const [adapterEntryPda] = findPda(
      [Buffer.from("adapter_entry"), adapterProgram.publicKey.toBuffer()],
      PROGRAM_IDS.adapterRegistry
    );

    const ix = buildProposeAdapter(
      {
        proposer: authority.publicKey,
        registryState: registryStatePda,
        adapterEntry: adapterEntryPda,
        adapterProgram: adapterProgram.publicKey,
        underlyingMint: underlyingMint.publicKey,
        rent: RENT_SYSVAR,
        systemProgram: SystemProgram.programId,
      },
      "Test Adapter",
      "https://example.com/metadata.json",
    );
    await provider.sendIx(ix);

    const info = await provider.connection.getAccountInfo(adapterEntryPda);
    expect(info).to.not.be.null;
    const entry = deserializeAdapterEntry(info!.data);
    expect(entry.name).to.equal("Test Adapter");
    expect(entry.status).to.equal(0);
    expect(entry.adapterProgramId.toString()).to.equal(
      adapterProgram.publicKey.toString()
    );

    const stateInfo = await provider.connection.getAccountInfo(registryStatePda);
    const state = deserializeRegistryState(stateInfo!.data);
    expect(state.totalProposed).to.be.greaterThan(0);
  });

  it("approves a proposed adapter", async () => {
    const adapterProgram = Keypair.generate();
    const underlyingMint = Keypair.generate();

    const [adapterEntryPda] = findPda(
      [Buffer.from("adapter_entry"), adapterProgram.publicKey.toBuffer()],
      PROGRAM_IDS.adapterRegistry
    );

    await provider.sendIx(
      buildProposeAdapter(
        {
          proposer: authority.publicKey,
          registryState: registryStatePda,
          adapterEntry: adapterEntryPda,
          adapterProgram: adapterProgram.publicKey,
          underlyingMint: underlyingMint.publicKey,
          rent: RENT_SYSVAR,
          systemProgram: SystemProgram.programId,
        },
        "Approved Adapter",
        "https://example.com/meta.json",
      )
    );

    await provider.sendIx(
      buildApproveAdapter(
        authority.publicKey,
        registryStatePda,
        adapterEntryPda,
      )
    );

    const info = await provider.connection.getAccountInfo(adapterEntryPda);
    const entry = deserializeAdapterEntry(info!.data);
    expect(entry.status).to.equal(1);
    expect(entry.approvedAt).to.be.greaterThan(0);
  });

  it("revokes an approved adapter", async () => {
    const adapterProgram = Keypair.generate();
    const underlyingMint = Keypair.generate();

    const [adapterEntryPda] = findPda(
      [Buffer.from("adapter_entry"), adapterProgram.publicKey.toBuffer()],
      PROGRAM_IDS.adapterRegistry
    );

    await provider.sendIx(
      buildProposeAdapter(
        {
          proposer: authority.publicKey,
          registryState: registryStatePda,
          adapterEntry: adapterEntryPda,
          adapterProgram: adapterProgram.publicKey,
          underlyingMint: underlyingMint.publicKey,
          rent: RENT_SYSVAR,
          systemProgram: SystemProgram.programId,
        },
        "Revoke Target",
        "https://example.com/meta.json",
      )
    );

    await provider.sendIx(
      buildApproveAdapter(
        authority.publicKey,
        registryStatePda,
        adapterEntryPda,
      )
    );

    await provider.sendIx(
      buildRevokeAdapter(
        authority.publicKey,
        registryStatePda,
        adapterEntryPda,
      )
    );

    const info = await provider.connection.getAccountInfo(adapterEntryPda);
    const entry = deserializeAdapterEntry(info!.data);
    expect(entry.status).to.equal(2);
    expect(entry.revokedAt).to.be.greaterThan(0);
  });

  it("rejects unauthorized approve attempts", async () => {
    const unauthorizedUser = Keypair.generate();
    await airdrop(provider.connection, unauthorizedUser.publicKey);

    const adapterProgram = Keypair.generate();
    const underlyingMint = Keypair.generate();

    const [adapterEntryPda] = findPda(
      [Buffer.from("adapter_entry"), adapterProgram.publicKey.toBuffer()],
      PROGRAM_IDS.adapterRegistry
    );

    await provider.sendIx(
      buildProposeAdapter(
        {
          proposer: authority.publicKey,
          registryState: registryStatePda,
          adapterEntry: adapterEntryPda,
          adapterProgram: adapterProgram.publicKey,
          underlyingMint: underlyingMint.publicKey,
          rent: RENT_SYSVAR,
          systemProgram: SystemProgram.programId,
        },
        "Unauth Test",
        "https://example.com/meta.json",
      )
    );

    try {
      const unauthorizedProvider = new TestProvider(provider.connection, unauthorizedUser);
      const ix = buildApproveAdapter(
        unauthorizedUser.publicKey,
        registryStatePda,
        adapterEntryPda,
      );
      await unauthorizedProvider.sendIx(ix);
      expect.fail("Should have thrown unauthorized error");
    } catch (err: any) {
      expect(err.toString()).to.contain("custom program error");
    }
  });

  it("transfers governance", async () => {
    const newAuthority = Keypair.generate();

    await provider.sendIx(
      buildTransferGovernance(
        authority.publicKey,
        registryStatePda,
        newAuthority.publicKey,
      )
    );

    const info = await provider.connection.getAccountInfo(registryStatePda);
    const state = deserializeRegistryState(info!.data);
    expect(state.authority.toString()).to.equal(
      newAuthority.publicKey.toString()
    );

    await airdrop(provider.connection, newAuthority.publicKey);

    const newAuthorityProvider = new TestProvider(provider.connection, newAuthority);
    await newAuthorityProvider.sendIx(
      buildTransferGovernance(
        newAuthority.publicKey,
        registryStatePda,
        authority.publicKey,
      )
    );
  });
});
