import { Codama, updateInstructionsVisitor } from 'codama';

/**
 * Removes the internal emitEvent instruction from client APIs.
 */
export function removeEmitInstruction(rewardsCodama: Codama): Codama {
    rewardsCodama.update(
        updateInstructionsVisitor({
            emitEvent: {
                delete: true,
            },
        }),
    );
    return rewardsCodama;
}
