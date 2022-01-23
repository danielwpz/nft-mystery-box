import { createContract, createWorkspace } from "./test_helper";
import Big from "big.js";
import { NEAR } from "near-workspaces-ava";

const workspace = createWorkspace(
    10,
    {
        'r1.near': 4000, // 40% of royalty
        'r2.near': 6000, // 60% of royalty
    },
    1000 // 10% of total income goes to royalty
);

workspace.test('empty royalty', async (test, {alice, root}) => {
    const contract = await createContract(root, 'payout1');
    const deposit = Big(await contract.view('cost_for', { n: 1 }));
    const tokens = await alice.call(
        contract,
        'buy',
        { n: 1 },
        {
            attachedDeposit: deposit.toFixed(0)
        }
    );
    const tokenId = tokens[0].token_id;

    const result = await contract.view(
        'nft_payout',
        {
            token_id: tokenId,
            balance: Big(1000).toFixed(0)
        }
    );
    test.is(
        Object.keys(result.payout).length,
        0,
        'expect empty payout with no royalty'
    );
});

workspace.test('payout calculation', async (test, {alice, contract}) => {
    const deposit = Big(await contract.view('cost_for', { n: 1 }));
    const tokens = await alice.call(
        contract,
        'buy',
        { n: 1 },
        {
            attachedDeposit: deposit.toFixed(0)
        }
    );
    const tokenId = tokens[0].token_id;

    const amount = Big(NEAR.parse('100').toString(10));
    const result = await contract.view(
        'nft_payout',
        {
            token_id: tokenId,
            balance: amount.toFixed(0)
        }
    );

    const payouts = result.payout;

    test.is(
        payouts['r1.near'],
        amount.mul(0.1).mul(0.4).toFixed(0),
        'r1.near payout wrong'
    );
    test.is(
        payouts['r2.near'],
        amount.mul(0.1).mul(0.6).toFixed(0),
        'r1.near payout wrong'
    );
    test.is(
        payouts[alice.accountId],
        amount.mul(0.9).toFixed(0),
        'alice payout wrong'
    );
});
