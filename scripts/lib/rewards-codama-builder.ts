import { Codama, createFromJson } from 'codama';
import {
    appendAccountDiscriminator,
    appendAccountVersion,
    appendPdaDerivers,
    setInstructionAccountDefaultValues,
    updateInstructionBumps,
} from './updates/';
import { removeEmitInstruction } from './updates/remove-emit-instruction';

/**
 * Builder for applying Codama IDL transformations before client generation.
 */
export class RewardsCodamaBuilder {
    private codama: Codama;

    constructor(rewardsIdl: any) {
        const idlJson = typeof rewardsIdl === 'string' ? rewardsIdl : JSON.stringify(rewardsIdl);
        this.codama = createFromJson(idlJson);
    }

    appendAccountDiscriminator(): this {
        this.codama = appendAccountDiscriminator(this.codama);
        return this;
    }

    appendAccountVersion(): this {
        this.codama = appendAccountVersion(this.codama);
        return this;
    }

    appendPdaDerivers(): this {
        this.codama = appendPdaDerivers(this.codama);
        return this;
    }

    setInstructionAccountDefaultValues(): this {
        this.codama = setInstructionAccountDefaultValues(this.codama);
        return this;
    }

    updateInstructionBumps(): this {
        this.codama = updateInstructionBumps(this.codama);
        return this;
    }

    removeEmitInstruction(): this {
        this.codama = removeEmitInstruction(this.codama);
        return this;
    }

    build(): Codama {
        return this.codama;
    }
}

export function createRewardsCodamaBuilder(rewardsIdl: any): RewardsCodamaBuilder {
    return new RewardsCodamaBuilder(rewardsIdl);
}
