import { Codama, bottomUpTransformerVisitor, structFieldTypeNode, numberTypeNode, assertIsNode, isNode } from 'codama';

/**
 * Adds discriminator fields to account structs for type-safe deserialization.
 */
export function appendAccountDiscriminator(rewardsCodama: Codama): Codama {
    rewardsCodama.update(
        bottomUpTransformerVisitor([
            {
                select: '[accountNode]',
                transform: node => {
                    assertIsNode(node, 'accountNode');

                    if (isNode(node.data, 'structTypeNode')) {
                        const updatedNode = {
                            ...node,
                            data: {
                                ...node.data,
                                fields: [
                                    structFieldTypeNode({
                                        name: 'discriminator',
                                        type: numberTypeNode('u8'),
                                    }),
                                    ...node.data.fields,
                                ],
                            },
                        };

                        if (node.size !== undefined) {
                            return {
                                ...updatedNode,
                                size: (node.size ?? 0) + 1,
                            };
                        }

                        return updatedNode;
                    }

                    return node;
                },
            },
        ]),
    );
    return rewardsCodama;
}
