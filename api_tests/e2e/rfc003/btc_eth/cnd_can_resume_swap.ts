import { sleep } from "../../../lib/util";
import { createActors } from "../../../lib_sdk/create_actors";

setTimeout(function() {
    describe("cnd can resume swap", function() {
        this.timeout(60000);
        it("with alice down and bob funds", async function() {
            const { alice, bob } = await createActors(
                "resume_alice_down_during_fund.log"
            );

            await alice.sendRequest();
            await bob.accept();

            await alice.fund();
            await alice.stop();

            // Action happens while alice is down.
            await bob.fund();

            // Blocks are geneated every second here, we to ensure
            // we look into the past for the transaction.
            await sleep(2000);
            await alice.start();

            await alice.redeem();
            await bob.redeem();

            await alice.assertSwapped();
            await bob.assertSwapped();
        });

        it("with bob down and alice funds", async function() {
            const { alice, bob } = await createActors(
                "resume_bob_down_during_fund.log"
            );

            await alice.sendRequest();
            await bob.accept();

            await bob.stop();

            // Action happens while bob is down.
            await alice.fund();

            // Blocks are geneated every second here, we to ensure
            // we look into the past for the transaction.
            await sleep(2000);
            await bob.start();
            await bob.fund();

            await alice.redeem();
            await bob.redeem();

            await alice.assertSwapped();
            await bob.assertSwapped();
        });

        it("with bob down and alice redeems", async function() {
            const { alice, bob } = await createActors(
                "resume_bob_down_during_redeem.log"
            );

            await alice.sendRequest();
            await bob.accept();

            await alice.fund();
            await bob.fund();

            await bob.stop();

            // Action happens while bob is down.
            await alice.redeem();

            // Blocks are geneated every second here, we to ensure
            // we look into the past for the transaction.
            await sleep(2000);
            await bob.start();

            await bob.redeem();

            await alice.assertSwapped();
            await bob.assertSwapped();
        });

        it("with alice down and bob redeems", async function() {
            const { alice, bob } = await createActors(
                "resume_alice_down_during_redeem.log"
            );

            await alice.sendRequest();
            await bob.accept();

            await alice.fund();
            await bob.fund();

            await alice.redeem();
            await alice.stop();

            // Action happens while alice is down.
            await bob.redeem();

            // Blocks are geneated every second here, we to ensure
            // we look into the past for the transaction.
            await sleep(2000);
            await alice.start();

            await alice.assertSwapped();
            await bob.assertSwapped();
        });
    });
    run();
}, 0);
