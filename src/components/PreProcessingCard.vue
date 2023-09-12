
<script>
import { ref } from 'vue'

export default {
    data() {
        return {
            preampMarkerLabel: val => `${Math.round(val * 100) / 100}dB`
        }
    },
    props: {
        preamp: ref(0),
        reverse_stereo: ref(false),
        expansion: ref(Boolean)
    },
    emits: ['update:preamp', 'update:reverse_stereo', 'update:expansion']
}
</script>
<template>
    <q-card flat bordered class="q-mx-none">
        <q-expansion-item default-opened expand-separator :model-value="expansion"
            @update:model-value="(value) => $emit('update:expansion', value)" label="Input preprocessing"
            header-class="title-bar-lv1">
            <q-card-section class="q-pb-none">
                <div class="info-box">
                    These controls are used to preprocess audio samples before the rest of the audio processing takes place.
                    If you hear crackling noises while listening to music, it may be because the parametric filters are
                    boosting the volume at certain frequencies. This can cause the filtered samples to be clipped at a
                    maximum value. Reducing the input volume with the preamplifier can help alleviate the issue.
                </div>
            </q-card-section>
            <q-card-section>
                <q-item>
                    <q-item-section>
                        <div class="row justify-start items-center q-gutter-sm">
                            <q-chip icon="volume_up" class="control-label" color=secondary text-color=white>PreAmp</q-chip>
                        </div>
                        <q-slider :model-value="preamp" @update:model-value="(value) => $emit('update:preamp', value)"
                            :min="-10" :max="10" :step="0.1" :markers="2" :marker-labels="preampMarkerLabel"
                            :label-value="preamp + 'dB'" label />
                    </q-item-section>
                </q-item>
                <q-checkbox label="Reverse Stereo" :model-value="reverse_stereo"
                    @update:model-value="(value) => $emit('update:reverse_stereo', value)" />
            </q-card-section>
        </q-expansion-item>
    </q-card>
</template>