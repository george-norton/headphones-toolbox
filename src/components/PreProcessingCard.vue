
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
        reverseStereo: ref(false)
    },
    emits: ['update:preamp', 'update:reverseStereo']
}
</script>
<template>
    <q-card flat bordered class="q-mx-none">
        <q-card-section class="title-bar-lv1 q-py-sm">
            <div class="text-h6">Input preprocessing</div>
        </q-card-section>
        <q-card-section class="q-pb-none">
            <div class="info-box">
                These controls are used to preprocess audio samples before the rest of the audio processing takes place. The
                preamplifier is particularly useful as it is likely that the parametric filters will increase the magnitude
                of the audio samples at specific frequencies. When this happens the audio samples can get clipped at a
                maximum value which leads to audio crackling. Reducing the input sample volume with the preamplifier can
                avoid these artefacts.
            </div>
        </q-card-section>
        <q-card-section>
            <q-item>
                <q-item-section>
                    <div class="col">
                        <q-chip class="control-label" color=secondary text-color=white>PreAmp</q-chip>
                    </div>
                    <q-slider :model-value="preamp" @update:model-value="(value) => $emit('update:preamp', value)"
                        :min="-50" :max="50" :markers="10" :marker-labels="preampMarkerLabel" :label-value="preamp + '%'"
                        label />
                </q-item-section>
            </q-item>
        <q-checkbox label="Reverse Stereo" :model-value="reverseStereo"
            @update:model-value="(value) => $emit('update:reverseStereo', value)" />
    </q-card-section>
</q-card></template>