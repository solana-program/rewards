import { Codama, updateInstructionsVisitor, accountBumpValueNode } from 'codama';

/**
 * Sets default bump values for rewards program instructions.
 */
export function updateInstructionBumps(rewardsCodama: Codama): Codama {
    rewardsCodama.update(
        updateInstructionsVisitor({
            createDistribution: {
                arguments: {
                    bump: {
                        defaultValue: accountBumpValueNode('distributionConfig'),
                    },
                },
            },
        }),
    );
    return rewardsCodama;
}
