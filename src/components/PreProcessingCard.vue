
<script>
import { reactive, ref } from 'vue'

export default {
    data() {
        return {
            preampMarkerLabel: val => `${val}%`
        }
    },
    props: {
        preamp: ref(0),
        reverseStereo: ref(false),
        expansion: ref(Boolean)
    },
    emits: ['update:preamp', 'update:reverseStereo', 'update:expansion']
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
                            :min="-50" :max="50" :markers="10" :marker-labels="preampMarkerLabel"
                            :label-value="preamp + '%'" label />
                    </q-item-section>
                </q-item>
                <q-checkbox label="Reverse Stereo" :model-value="reverseStereo"
                    @update:model-value="(value) => $emit('update:reverseStereo', value)" />
            </q-card-section>
        </q-expansion-item>
    </q-card>
</template>