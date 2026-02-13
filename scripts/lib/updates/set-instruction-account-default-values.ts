import {
    Codama,
    pdaNode,
    pdaValueNode,
    pdaSeedValueNode,
    publicKeyTypeNode,
    accountValueNode,
    variablePdaSeedNode,
    publicKeyValueNode,
    setInstructionAccountDefaultValuesVisitor,
} from 'codama';

const REWARDS_PROGRAM_ID = 'REWArDioXgQJ2fZKkfu9LCLjQfRwYWVVfsvcsR5hoXi';

const ATA_PROGRAM_ID = 'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL';
const SYSTEM_PROGRAM_ID = '11111111111111111111111111111111';
const TOKEN_PROGRAM_ID = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA';

function createAtaPdaValueNode(ownerAccount: string, mintAccount: string, tokenProgram: string) {
    return pdaValueNode(
        pdaNode({
            name: 'associatedTokenAccount',
            seeds: [
                variablePdaSeedNode('owner', publicKeyTypeNode()),
                variablePdaSeedNode('tokenProgram', publicKeyTypeNode()),
                variablePdaSeedNode('mint', publicKeyTypeNode()),
            ],
            programId: ATA_PROGRAM_ID,
        }),
        [
            pdaSeedValueNode('owner', accountValueNode(ownerAccount)),
            pdaSeedValueNode('tokenProgram', accountValueNode(tokenProgram)),
            pdaSeedValueNode('mint', accountValueNode(mintAccount)),
        ],
    );
}

/**
 * Sets default values for common instruction accounts (program IDs, PDAs, ATAs).
 */
export function setInstructionAccountDefaultValues(rewardsCodama: Codama): Codama {
    rewardsCodama.update(
        setInstructionAccountDefaultValuesVisitor([
            // Global Constants
            {
                account: 'rewardsProgram',
                defaultValue: publicKeyValueNode(REWARDS_PROGRAM_ID),
            },
            {
                account: 'tokenProgram',
                defaultValue: publicKeyValueNode(TOKEN_PROGRAM_ID),
            },
            {
                account: 'associatedTokenProgram',
                defaultValue: publicKeyValueNode(ATA_PROGRAM_ID),
            },
            {
                account: 'systemProgram',
                defaultValue: publicKeyValueNode(SYSTEM_PROGRAM_ID),
            },
        ]),
    );
    return rewardsCodama;
}
