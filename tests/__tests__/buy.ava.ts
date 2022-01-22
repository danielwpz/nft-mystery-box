import { createWorkspace, assertFailed, ONE_NEAR } from './test_helper';
import Big from 'big.js';

const workspace = createWorkspace();

workspace.test('mint without deposit', async (test, {alice, contract}) => {
  await assertFailed(
    alice.call(contract, 'buy', { n: 1 }),
    'E03: no enough deposit'
  );
});

workspace.test('mint with just minting fee', async (_, {alice, contract}) => {
  const unitPrice = Big(await contract.view('unit_price'));
  const deposit = unitPrice.times(3)
  await assertFailed(
    alice.call(
      contract,
      'buy',
      { n: 3 },
      {
        attachedDeposit: deposit.toFixed(0)
      }
    ),
    'E03: no enough deposit'
  );
});

workspace.test('mint with exact fee', async (test, {alice, contract}) => {
  const n = 3;
  const cost = Big(await contract.view('cost_for', { n }));

  const tokens: [any] = await alice.call(
    contract,
    'buy',
    { n },
    {
      attachedDeposit: cost.toFixed(0)
    }
  );

  test.is(n, tokens.length);
});

workspace.test('mint with extra deposit', async (test, {alice, contract}) => {
  const n = 3;
  const cost = Big(await contract.view('cost_for', { n }));
  const deposit = cost.plus(ONE_NEAR) // one extra NEAR deposit

  const oldBalance = Big(
    (await alice.availableBalance()).toBigInt()
  );

  const tokens: [any] = await alice.call(
    contract,
    'buy',
    { n },
    {
      attachedDeposit: deposit.toFixed(0)
    }
  );
  test.is(n, tokens.length);

  const newBalance = Big(
    (await alice.availableBalance()).toBigInt()
  );

  const actualPaid = oldBalance.minus(newBalance);
  const refunded = deposit.minus(actualPaid); // 1 N + storage
  test.true(
    refunded.gt(ONE_NEAR),
    'refund less than 1 extra NEAR'
  );
});

workspace.test('pay 1 yN less than required', async (test, {alice, contract}) => {
  const n = 3;
  const cost = Big(await contract.view('cost_for', { n }));
  const deposit = cost.minus(1);

  await assertFailed(
    alice.call(
      contract,
      'buy',
      { n },
      {
        attachedDeposit: deposit.toFixed(0)
      }
    ),
    'E03: no enough deposit'
  );
});
