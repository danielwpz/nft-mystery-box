import { createContract, createWorkspace, ONE_NEAR } from "./test_helper";
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

workspace.test('income distribution', async (test, {alice, root}) => {
    // create a new contract and buy some nfts
    const lp1 = await root.createAccount('lp1');
    const lp2 = await root.createAccount('lp2');
    const lp3 = await root.createAccount('lp3');

    const royalties = {};
    royalties[lp1.accountId] = 2000;
    royalties[lp2.accountId] = 3000;
    royalties[lp3.accountId] = 5000;

    const contract = await createContract(
        root,
        'income1',
        10,
        royalties,
        1000
    );

    const deposit = Big(await contract.view('cost_for', { n: 2 }));
    await alice.call(
        contract,
        'buy',
        { n: 2 },
        {
            attachedDeposit: deposit.toFixed(0)
        }
    );

    // verify distribution
    const lps = [
        lp1, lp2, lp3
    ];
    const initBalance = {};
    for (const lp of lps) {
        initBalance[lp.accountId] = Big(
            (await lp.availableBalance()).toBigInt()
        );
    }

    await alice.call(
        contract,
        'distribute_income',
        {}
    );

    const totalIncome = ONE_NEAR.mul(2);

    for (const lp of lps) {
        const newBalance = Big(
            (await lp.availableBalance()).toBigInt()
        );

        const income = newBalance.minus(initBalance[lp.accountId]);
        const target = totalIncome
            .mul(royalties[lp.accountId])
            .div(10000);
        test.true(
            income.eq(target),
            `lp should receive ${target}, but got ${income}`
        );
    }
});
