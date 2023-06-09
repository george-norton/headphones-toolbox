<script>
import FilterVue from './Filter.vue'
import { ref } from 'vue'

export default {
    data() {
        return {
            overSamplingOptions: [{ label: "x16", value: 0 }, { label: "x32", value: 1 }],
            phaseOptions: [{ label: "Normal", value: 0 }, { label: "Inverted", value: 1 }],
            rolloffOptions: [{ label: "Sharp", value: 0 }, { label: "Slow", value: 1 }],
            deEmphasisOptions: [{ label: "Disabled", value: 0 }, { label: "Enabled", value: 1 }]
        }
    },
    props: {
        oversampling: ref(Number),
        phase: ref(Number),
        rolloff: ref(Number),
        de_emphasis: ref(Number),
        expansion: ref(Boolean)
    },
    emits: ['update:oversampling', 'update:phase', 'update:rolloff', 'update:de_emphasis', 'update:expansion'],
    methods: {
    }
}
</script>
<template>
    <q-card flat bordered class="q-mx-none">
        <q-expansion-item default-opened expand-separator :model-value="expansion"
            @update:model-value="(value) => $emit('update:expansion', value)" label="Codec configuration"
            caption="Not implemented yet!" header-class="title-bar-lv1">
            <q-card-section class="q-pb-none">
                <div class="info-box">
                    The Texas Instruments PCM3060 codec chip is used as a digital to analog converter in the DAC. It has
                    a few additional filtering capabilities which can be configured here.
                </div>
            </q-card-section>
            <q-card-section class="row q-gutter-md">
                <q-select :model-value="oversampling" :options="overSamplingOptions" map-options
                    @update:model-value="(value) => $emit('update:oversampling', value)" label="Oversampling" class="col" />
                <q-select :model-value="phase" :options="phaseOptions" map-options
                    @update:model-value="(value) => $emit('update:phase', value)" label="Output Phase" class="col" />
                <q-select :model-value="rolloff" :options="rolloffOptions" map-options
                    @update:model-value="(value) => $emit('update:rolloff', value)" label="Digital Filter Rolloff"
                    class="col" />
                <q-select :model-value="de_emphasis" :options="deEmphasisOptions" map-options
                    @update:model-value="(value) => $emit('update:de_emphasis', value)" label="Digital De-Emphasis"
                    class="col" />
            </q-card-section>
        </q-expansion-item>
    </q-card>
</template>