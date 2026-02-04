import {
    Codama,
    constantPdaSeedNode,
    stringTypeNode,
    stringValueNode,
    variablePdaSeedNode,
    publicKeyTypeNode,
    addPdasVisitor,
} from 'codama';

/**
 * Adds PDA derivation functions for rewards program accounts.
 */
export function appendPdaDerivers(rewardsCodama: Codama): Codama {
    rewardsCodama.update(
        addPdasVisitor({
            rewardsProgram: [
                {
                    name: 'distributionConfig',
                    seeds: [
                        constantPdaSeedNode(stringTypeNode('utf8'), stringValueNode('distribution_config')),
                        variablePdaSeedNode('configSeed', publicKeyTypeNode()),
                    ],
                },
                {
                    name: 'eventAuthority',
                    seeds: [constantPdaSeedNode(stringTypeNode('utf8'), stringValueNode('event_authority'))],
                },
            ],
        }),
    );
    return rewardsCodama;
}
