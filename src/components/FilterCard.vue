
<script>
import FilterVue from './Filter.vue'
import { reactive, ref } from 'vue'

const STEPS = 4096
export default {
    data() {
        return {
        }
    },
    props: {
        filters: reactive([])
    },
    methods: {
        addFilter() {
            this.filters.push({ type: "lowpass", q: 0, f0: 0, dbGain: 0, enabled: true })
        },
        deleteFilter(filter) {
            for (var i = 0; i < this.filters.length; i++) {
                if (this.filters[i] == filter) {
                    this.filters.splice(i, 1)
                    break
                }
            }
        }
    },
    components: {
        FilterVue
    }
}
</script>

<template>
    <q-card flat bordered class="q-mx-none">
        <q-card-section class="bg-grey-4 q-py-sm">
            <div class="text-h6">Input preprocessing</div>
        </q-card-section>
        <q-card-section>
            Nothing yet. Coming soon - PreAmp, Reverse Stereo.
        </q-card-section>
    </q-card>
    <q-card flat bordered class="q-mx-none">
        <q-card-section class="bg-grey-4 q-py-sm">
            <div class="text-h6">Parametric filters</div>
        </q-card-section>
        <q-card-section>
            <q-list class="col-12">
                <q-item style="padding-left:0px; padding-right:0px" v-for="filter in filters">
                    <FilterVue v-model:type="filter.type" v-model:f0="filter.f0" v-model:dbGain="filter.dbGain"
                        v-model:q="filter.q" v-model:enabled="filter.enabled" @delete:filter="deleteFilter(filter)"
                        ref="filter" />
                </q-item>
            </q-list>
            <div class="row">
                <q-btn fab icon="add" label="New Filter" color="primary" @click="addFilter()" />
            </div>
        </q-card-section>
    </q-card>
    <q-card flat bordered class="q-mx-none">
        <q-card-section class="bg-grey-4 q-py-sm">
            <div class="text-h6">Codec configuration</div>
        </q-card-section>
        <q-card-section>
            Nothing yet. Coming soon - configure the various filters provided by the TI3060 DAC chip.
        </q-card-section>
    </q-card>
</template>
