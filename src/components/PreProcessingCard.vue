
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
        <q-card-section class="bg-grey-4 q-py-sm">
            <div class="text-h6">Input preprocessing</div>
        </q-card-section>
        <q-card-section>
            <q-item>
                <q-item-section>
                    <b>PreAmp</b>
                    <q-slider :model-value="preamp" @update:model-value="(value) => $emit('update:preamp', value)" :min="-50" :max="50" :markers="10" :marker-labels="preampMarkerLabel"
                        :label-value="preamp + '%'" label />
                </q-item-section>
            </q-item>
            <q-checkbox label="Reverse Stereo" :model-value="reverseStereo" @update:model-value="(value) => $emit('update:reverseStereo', value)" />
        </q-card-section>
    </q-card>
</template>