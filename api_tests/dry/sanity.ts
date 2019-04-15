// These are stateless tests -- they don't require any state of the comit node and they don't change it
// They are mostly about checking invalid request responses
import { Actor } from "../lib/actor";
import * as chai from "chai";
import { HarnessGlobal, sleep } from "../lib/util";

import chaiHttp = require("chai-http");

chai.use(chaiHttp);
const should = chai.should();

declare var global: HarnessGlobal;

const alice = new Actor("alice", global.config, global.project_root);

// the `setTimeout` forces it to be added on the event loop
// This is needed because there is no async call in the test
// And hence it does not get run without this `setTimeout`
setTimeout(async function() {
    describe("Sanity tests", () => {
        it("[Alice] Returns 404 when you try and GET a non-existent swap", async () => {
            let res = await chai
                .request(alice.comit_node_url())
                .get("/swaps/rfc003/deadbeef-dead-beef-dead-deadbeefdead");

            res.should.have.status(404);
        });

        it("Returns a 404 for an action on a non-existent swap", async () => {
            let res = await chai
                .request(alice.comit_node_url())
                .post(
                    "/swaps/rfc003/deadbeef-dead-beef-dead-deadbeefdead/accept"
                )
                .send({});
            res.should.have.status(404);
        });

        it("Returns an empty list when calling GET /swaps when there are no swaps", async () => {
            let res = await chai.request(alice.comit_node_url()).get("/swaps");

            let swaps = res.body._embedded.swaps;
            swaps.should.be.an("array");
            swaps.should.have.lengthOf(0);
        });

        it("[Alice] Returns 400 Swap not supported for an unsupported combination of parameters", async () => {
            let res = await chai
                .request(alice.comit_node_url())
                .post("/swaps/rfc003")
                .send({
                    alpha_ledger: {
                        name: "Thomas' wallet",
                    },
                    beta_ledger: {
                        name: "Higher-Dimension", // This is the coffee place downstairs
                    },
                    alpha_asset: {
                        name: "AUD",
                        quantity: "3.5",
                    },
                    beta_asset: {
                        name: "Espresso",
                        "double-shot": true,
                    },
                    alpha_ledger_refund_identity: "",
                    beta_ledger_redeem_identity: "",
                    alpha_expiry: 123456789,
                    beta_expiry: 123456789,
                    peer: "0.0.0.0",
                });
            res.should.have.status(400);
            res.body.title.should.equal("Swap not supported.");
        });

        it("[Alice] Returns 400 bad request for malformed requests", async () => {
            let res = await chai
                .request(alice.comit_node_url())
                .post("/swaps/rfc003")
                .send({
                    garbage: true,
                });
            res.should.have.status(400);
            res.body.title.should.equal("Bad Request");
        });

        it("[Alice] Should have no peers before making a swap request", async () => {
            let res = await chai.request(alice.comit_node_url()).get("/peers");
            res.should.have.status(200);
            res.body.peers.should.have.length(0);
        });
    });

    run();
}, 0);
