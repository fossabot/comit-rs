import { expect } from "chai";
import { BigNumber, Cnd, ComitClient, Swap } from "comit-sdk";
import { parseEther } from "ethers/utils";
import getPort from "get-port";
import { Logger } from "log4js";
import { E2ETestActorConfig } from "../../lib/config";
import { LedgerConfig } from "../../lib/ledger_runner";
import "../../lib/setup_chai";
import { Asset, AssetKind } from "../asset";
import { CndInstance } from "../cnd_instance";
import { Ledger, LedgerKind } from "../ledger";
import { sleep } from "../utils";
import { Wallet, Wallets } from "../wallets";
import { Actors } from "./index";

export class Actor {
    public static defaultActionConfig = {
        maxTimeoutSecs: 5,
        tryIntervalSecs: 1,
    };

    public static async newInstance(
        loggerFactory: (name: string) => Logger,
        name: string,
        ledgerConfig: LedgerConfig,
        projectRoot: string,
        logRoot: string
    ) {
        const actorConfig = new E2ETestActorConfig(
            await getPort(),
            await getPort(),
            name
        );

        const cndInstance = new CndInstance(
            projectRoot,
            logRoot,
            actorConfig,
            ledgerConfig
        );

        await cndInstance.start();

        const logger = loggerFactory(name);
        logger.level = "debug";

        logger.info(
            "Created new actor with config %s",
            JSON.stringify(actorConfig.generateCndConfigFile(ledgerConfig))
        );

        return new Actor(logger, cndInstance);
    }

    public actors: Actors;
    public wallets: Wallets;

    private comitClient: ComitClient;
    private readonly cnd: Cnd;
    private swap: Swap;

    private alphaLedger: Ledger;
    private alphaAsset: Asset;

    private betaLedger: Ledger;
    private betaAsset: Asset;

    private readonly startingBalances: Map<AssetKind, BigNumber>;
    private readonly expectedBalanceChanges: Map<AssetKind, BigNumber>;

    private constructor(
        private readonly logger: Logger,
        private readonly cndInstance: CndInstance
    ) {
        this.wallets = new Wallets({});
        const { address, port } = cndInstance.getConfigFile().http_api.socket;
        this.cnd = new Cnd(`http://${address}:${port}`);

        this.startingBalances = new Map();
        this.expectedBalanceChanges = new Map();
    }

    public async sendRequest(
        maybeAlphaAssetKind?: AssetKind,
        maybeBetaAssetKind?: AssetKind
    ) {
        const alphaAssetKind = maybeAlphaAssetKind
            ? maybeAlphaAssetKind
            : this.defaultAlphaAssetKind();
        const betaAssetKind = maybeBetaAssetKind
            ? maybeBetaAssetKind
            : this.defaultBetaAssetKind();

        // By default, we will send the swap request to bob
        const to = this.actors.bob;

        this.logger.info("Sending swap request");

        this.alphaLedger = defaultLedgerDescriptionForAsset(alphaAssetKind);
        this.alphaAsset = defaultAssetDescriptionForAsset(alphaAssetKind);
        to.alphaLedger = this.alphaLedger;
        to.alphaAsset = this.alphaAsset;

        this.logger.debug(
            "Derived %o from asset %s",
            this.alphaLedger,
            alphaAssetKind
        );
        this.logger.debug(
            "Derived %o from asset %s",
            this.alphaAsset,
            alphaAssetKind
        );

        this.betaLedger = defaultLedgerDescriptionForAsset(betaAssetKind);
        this.betaAsset = defaultAssetDescriptionForAsset(betaAssetKind);
        to.betaLedger = this.betaLedger;
        to.betaAsset = this.betaAsset;

        this.logger.debug(
            "Derived %o from asset %s",
            this.betaLedger,
            betaAssetKind
        );
        this.logger.debug(
            "Derived %o from asset %s",
            this.betaAsset,
            betaAssetKind
        );

        await this.initializeDependencies();
        await to.initializeDependencies();

        await this.setStartingBalance([
            this.alphaAsset,
            { name: this.betaAsset.name, quantity: "0" },
        ]);
        await to.setStartingBalance([
            { name: to.alphaAsset.name, quantity: "0" },
            to.betaAsset,
        ]);

        this.expectedBalanceChanges.set(
            betaAssetKind,
            new BigNumber(this.betaAsset.quantity)
        );
        to.expectedBalanceChanges.set(
            alphaAssetKind,
            new BigNumber(to.alphaAsset.quantity)
        );

        const comitClient: ComitClient = this.getComitClient();

        const payload = {
            alpha_ledger: this.alphaLedger,
            beta_ledger: this.betaLedger,
            alpha_asset: this.alphaAsset,
            beta_asset: this.betaAsset,
            peer: {
                peer_id: await to.cnd.getPeerId(),
                address_hint: await to.cnd
                    .getPeerListenAddresses()
                    .then(addresses => addresses[0]),
            },
            ...(await this.additionalIdentities(alphaAssetKind, betaAssetKind)),
            ...defaultExpiryTimes(),
        };

        this.swap = await comitClient.sendSwap(payload);
        to.swap = new Swap(
            to.wallets.bitcoin.inner,
            to.wallets.ethereum.inner,
            to.cnd,
            this.swap.self
        );
        this.logger.debug("Created new swap at %s", this.swap.self);
    }

    public async accept() {
        if (!this.swap) {
            throw new Error("Cannot accept inexistent swap");
        }

        await this.swap.accept(Actor.defaultActionConfig);
    }

    public async fund() {
        if (!this.swap) {
            throw new Error("Cannot fund inexistent swap");
        }

        const txid = await this.swap.fund(Actor.defaultActionConfig);
        this.logger.debug("Funded swap %s in %s", this.swap.self, txid);

        const entity = await this.swap.fetchDetails();
        switch (entity.properties.role) {
            case "Alice":
                await this.actors.alice.assertAlphaFunded();
                await this.actors.bob.assertAlphaFunded();
                break;
            case "Bob":
                await this.actors.alice.assertBetaFunded();
                await this.actors.bob.assertBetaFunded();
                break;
        }
    }

    public async refund() {
        if (!this.swap) {
            throw new Error("Cannot refund inexistent swap");
        }

        const entity = await this.swap.fetchDetails();

        switch (entity.properties.role) {
            case "Alice":
                await this.waitForAlphaExpiry();
                break;
            case "Bob":
                await this.waitForBetaExpiry();
                break;
        }

        const txid = await this.swap.refund(Actor.defaultActionConfig);
        this.logger.debug("Refunded swap %s in %s", this.swap.self, txid);

        switch (entity.properties.role) {
            case "Alice":
                await this.actors.alice.assertAlphaRefunded();
                await this.actors.bob.assertAlphaRefunded();
                break;
            case "Bob":
                await this.actors.alice.assertBetaRefunded();
                await this.actors.bob.assertBetaRefunded();
                break;
        }
    }

    public async redeem() {
        if (!this.swap) {
            throw new Error("Cannot redeem inexistent swap");
        }

        const txid = await this.swap.redeem(Actor.defaultActionConfig);
        this.logger.debug("Redeemed swap %s in %s", this.swap.self, txid);

        const entity = await this.swap.fetchDetails();
        switch (entity.properties.role) {
            case "Alice":
                await this.actors.alice.assertBetaRedeemed();
                await this.actors.bob.assertBetaRedeemed();
                break;
            case "Bob":
                await this.actors.alice.assertAlphaRedeemed();
                await this.actors.bob.assertAlphaRedeemed();
                break;
        }
    }

    public async currentSwapIsAccepted() {
        let swapEntity;

        do {
            swapEntity = await this.swap.fetchDetails();

            await sleep(200);
        } while (
            swapEntity.properties.state.communication.status !== "ACCEPTED"
        );
    }

    public async assertSwapped() {
        this.logger.debug("Checking if cnd reports status 'SWAPPED'");

        while (true) {
            await sleep(200);
            const entity = await this.swap.fetchDetails();
            if (entity.properties.status === "SWAPPED") {
                break;
            }
        }

        for (const [
            assetKind,
            expectedBalanceChange,
        ] of this.expectedBalanceChanges.entries()) {
            this.logger.debug(
                "Checking that %s balance changed by %d",
                assetKind,
                expectedBalanceChange
            );

            const wallet = this.wallets[
                defaultLedgerDescriptionForAsset(assetKind).name
            ];
            const expectedBalance = new BigNumber(
                this.startingBalances.get(assetKind)
            ).plus(expectedBalanceChange);
            const maximumFee = wallet.MaximumFee;

            const balanceInclFees = expectedBalance.minus(maximumFee);
            const currentWalletBalance = await wallet.getBalance();

            expect(currentWalletBalance).to.be.bignumber.gte(balanceInclFees);
        }
    }

    public async assertRefunded() {
        this.logger.debug("Checking if swap @ %s was refunded", this.swap.self);

        for (const [assetKind] of this.startingBalances.entries()) {
            const wallet = this.wallets[
                defaultLedgerDescriptionForAsset(assetKind).name
            ];
            const maximumFee = wallet.MaximumFee;

            this.logger.debug(
                "Checking that %s balance changed by max %d (MaximumFee)",
                assetKind,
                maximumFee
            );
            const expectedBalance = new BigNumber(
                this.startingBalances.get(assetKind)
            );
            const currentWalletBalance = await wallet.getBalance();
            const balanceInclFees = expectedBalance.minus(maximumFee);
            expect(currentWalletBalance).to.be.bignumber.gte(balanceInclFees);
        }
    }

    public async assertAlphaFunded() {
        await this.assertLedgerState("alpha_ledger", "FUNDED");
    }

    public async assertBetaFunded() {
        await this.assertLedgerState("beta_ledger", "FUNDED");
    }

    public async assertAlphaRedeemed() {
        await this.assertLedgerState("alpha_ledger", "REDEEMED");
    }

    public async assertBetaRedeemed() {
        await this.assertLedgerState("beta_ledger", "REDEEMED");
    }

    public async assertAlphaRefunded() {
        await this.assertLedgerState("alpha_ledger", "REFUNDED");
    }

    public async assertBetaRefunded() {
        await this.assertLedgerState("beta_ledger", "REFUNDED");
    }

    public async restart() {
        this.cndInstance.stop();
        await this.cndInstance.start();
    }

    public stop() {
        this.cndInstance.stop();
    }

    private async waitForAlphaExpiry() {
        const swapDetails = await this.swap.fetchDetails();

        const expiry = swapDetails.properties.state.communication.alpha_expiry;
        const wallet = this.wallets.getWalletForLedger(this.alphaLedger.name);

        await this.waitForExpiry(wallet, expiry);
    }

    private async waitForBetaExpiry() {
        const swapDetails = await this.swap.fetchDetails();

        const expiry = swapDetails.properties.state.communication.beta_expiry;
        const wallet = this.wallets.getWalletForLedger(this.betaLedger.name);

        await this.waitForExpiry(wallet, expiry);
    }

    private async waitForExpiry(wallet: Wallet, expiry: number) {
        let currentBlockchainTime = await wallet.getBlockchainTime();

        this.logger.debug(
            `Current blockchain time is ${currentBlockchainTime}`
        );

        let diff = expiry - currentBlockchainTime;

        if (diff > 0) {
            this.logger.debug(`Waiting for blockchain time to pass ${expiry}`);

            while (diff > 0) {
                await sleep(1000);

                currentBlockchainTime = await wallet.getBlockchainTime();
                diff = expiry - currentBlockchainTime;

                this.logger.debug(
                    `Current blockchain time is ${currentBlockchainTime}`
                );
            }
        }
    }

    private async assertLedgerState(
        ledger: string,
        status:
            | "NOT_DEPLOYED"
            | "DEPLOYED"
            | "FUNDED"
            | "REDEEMED"
            | "REFUNDED"
            | "INCORRECTLY_FUNDED"
    ) {
        this.logger.debug(
            "Waiting for cnd to see %s in state %s for swap @ %s",
            ledger,
            status,
            this.swap.self
        );

        let swapEntity;

        do {
            swapEntity = await this.swap.fetchDetails();

            await sleep(200);
        } while (swapEntity.properties.state[ledger].status !== status);

        this.logger.debug(
            "cnd saw %s in state %s for swap @ %s",
            ledger,
            status,
            this.swap.self
        );
    }

    private async additionalIdentities(
        alphaAsset: AssetKind,
        betaAsset: AssetKind
    ) {
        if (alphaAsset === "bitcoin" && betaAsset === "ether") {
            return {
                beta_ledger_redeem_identity: this.wallets.ethereum.account(),
            };
        }

        return {};
    }

    private async initializeDependencies() {
        for (const ledgerName of [
            this.alphaLedger.name,
            this.betaLedger.name,
        ]) {
            await this.wallets.initializeForLedger(ledgerName);
        }

        this.comitClient = new ComitClient(
            this.wallets.getWalletForLedger("bitcoin").inner,
            this.wallets.getWalletForLedger("ethereum").inner,
            this.cnd
        );
    }

    private getComitClient(): ComitClient {
        if (!this.comitClient) {
            throw new Error("ComitClient is not initialised");
        }

        return this.comitClient;
    }

    private async setStartingBalance(assets: Asset[]) {
        for (const asset of assets) {
            if (parseFloat(asset.quantity) === 0) {
                this.startingBalances.set(asset.name, new BigNumber(0));
                continue;
            }

            const ledger = defaultLedgerDescriptionForAsset(asset.name).name;

            this.logger.debug("Minting %s on %s", asset.name, ledger);
            await this.wallets.getWalletForLedger(ledger).mint(asset);

            const balance = await this.wallets[
                defaultLedgerDescriptionForAsset(asset.name).name
            ].getBalance();

            this.logger.debug(
                "Starting %s balance: ",
                asset.name,
                balance.toString()
            );
            this.startingBalances.set(asset.name, balance);
        }
    }

    private defaultAlphaAssetKind() {
        const defaultAlphaAssetKind = AssetKind.Bitcoin;
        this.logger.info(
            "AssetKind for alpha ledger not specified, defaulting to %s",
            defaultAlphaAssetKind
        );

        return defaultAlphaAssetKind;
    }

    private defaultBetaAssetKind() {
        const defaultBetaAssetKind = AssetKind.Ether;
        this.logger.info(
            "AssetKind for beta ledger not specified, defaulting to %s",
            defaultBetaAssetKind
        );

        return defaultBetaAssetKind;
    }
}

function defaultLedgerDescriptionForAsset(asset: AssetKind): Ledger {
    switch (asset) {
        case AssetKind.Bitcoin: {
            return {
                name: LedgerKind.Bitcoin,
                network: "regtest",
            };
        }
        case AssetKind.Ether: {
            return {
                name: LedgerKind.Ethereum,
                chain_id: 17,
            };
        }
    }
}

function defaultAssetDescriptionForAsset(asset: AssetKind): Asset {
    switch (asset) {
        case AssetKind.Bitcoin: {
            return {
                name: AssetKind.Bitcoin,
                quantity: "100000000",
            };
        }
        case AssetKind.Ether: {
            return {
                name: AssetKind.Ether,
                quantity: parseEther("10").toString(),
            };
        }
    }
}

function defaultExpiryTimes() {
    const alphaExpiry = Math.round(Date.now() / 1000) + 8;
    const betaExpiry = Math.round(Date.now() / 1000) + 3;

    return {
        alpha_expiry: alphaExpiry,
        beta_expiry: betaExpiry,
    };
}
