import { Workspace, NEAR } from 'near-workspaces-ava';
import assert from 'assert';
import Big from 'big.js';

export const ONE_NEAR = Big(NEAR.parse('1').toBigInt() as any);
type Royalty = {[account: string]: number};

export function createWorkspace(
  len = 10,
  royalties?: Royalty, 
  royaltyRate?: number
) {
  return Workspace.init(async ({ root }) => {
    const alice = await root.createAccount('alice');
    const contract = await createContract(
      root,
      'nft',
      len,
      royalties,
      royaltyRate
    );

    return { alice, contract };
  });
}

export async function createContract (
  root: any,
  name: string,
  len: number = 10, 
  royalties?: Royalty, 
  royaltyRate?: number
) {
  const args = {
    metadata: {
      spec: 'nft-1.0.0',
      name: 'test nft',
      symbol: 'nft'
    },
    len
  };

  if (royalties && royaltyRate) {
    args['royalties'] = royalties;
    args['royalty_rate'] = royaltyRate;
  }

  return root.createAndDeploy(
    name,
    'compiled-contracts/nft_mystery_box.wasm',
    {
      method: 'new',
      args
    },
  );
}

export async function assertFailed(promise: Promise<any>, msg?: string) {
  let failed = false;

  try {
    await promise;
  } catch (err) {
    failed = true;
    const errorMessage: string = err.message;
    if (msg) {
      assert(
        errorMessage.includes(msg),
        `Error message mismatch. expect: ${msg} , actual: ${errorMessage}`
      );
    }
  }

  assert(failed, `Transaction that should fail succeeded.`);
}
