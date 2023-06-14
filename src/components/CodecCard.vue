<script>
import FilterVue from './Filter.vue'
import { ref } from 'vue'

export default {
    data() {
        return {
            overSamplingOptions: [{ label: "x16", value: false }, { label: "x32", value: true }],
            phaseOptions: [{ label: "Normal", value: false }, { label: "Inverted", value: true }],
            rolloffOptions: [{ label: "Sharp", value: false }, { label: "Slow", value: true }],
            deEmphasisOptions: [{ label: "Disabled", value: false }, { label: "Enabled", value: true }]
        }
    },
    props: {
        oversampling: ref(Boolean),
        phase: ref(Boolean),
        rolloff: ref(Boolean),
        de_emphasis: ref(Boolean),
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
             header-class="title-bar-lv1">
            <q-card-section class="q-pb-none">
                <div class="info-box">
                    The Texas Instruments PCM3060 codec chip is used as a digital to analog converter in the DAC. It has
                    a few additional filtering capabilities which can be configured here. These options appear to have very
                    little effect.
                </div>
            </q-card-section>
            <q-card-section class="row q-gutter-md">
                <q-select :model-value="oversampling" :options="overSamplingOptions" map-options
                    @update:model-value="(value) => $emit('update:oversampling', value.value)" label="Oversampling" class="col" />
                <q-select :model-value="phase" :options="phaseOptions" map-options
                    @update:model-value="(value) => $emit('update:phase', value.value)" label="Output Phase" class="col" />
                <q-select :model-value="rolloff" :options="rolloffOptions" map-options
                    @update:model-value="(value) => $emit('update:rolloff', value.value)" label="Digital Filter Rolloff"
                    class="col" />
                <q-select :model-value="de_emphasis" :options="deEmphasisOptions" map-options
                    @update:model-value="(value) => $emit('update:de_emphasis', value.value)" label="Digital De-Emphasis"
                    class="col" />
            </q-card-section>
        </q-expansion-item>
    </q-card>
</template>