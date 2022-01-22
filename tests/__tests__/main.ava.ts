import { createWorkspace } from './test_helper';

const workspace = createWorkspace();

// test whether the contract is deployed successfully
workspace.test('is contract deployed', async (test, {alice, contract, root}) => {
  const contractMetadata = await contract.view('nft_metadata');

  test.is(
    contractMetadata.spec,
    'nft-1.0.0'
  );
  test.is(
    contractMetadata.name,
    'test nft'
  );
  test.is(
    contractMetadata.symbol,
    'nft'
  );
});
