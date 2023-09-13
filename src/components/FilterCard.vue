
<script>
import FilterVue from './Filter.vue'
import { ref } from 'vue'

export default {
    props: {
        filters: {
            type: Array,
            default: []
        },
        expansion: ref(Boolean)
    },
    methods: {
        addFilter() {
            this.filters.push({ filter_type: "peaking", q: 0.707106781, f0: 150, db_gain: 0, 
                                a0: 0.5, a1: 0.5, a2: 0.5, b0: 0.5, b1: 0.5, b2: 0.5, enabled: true })
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
    emits: ['update:expansion'],
    components: {
        FilterVue
    }
}
</script>

<template>
    <q-card flat bordered class="q-mx-none">
        <q-expansion-item default-opened expand-separator :model-value="expansion"
            @update:model-value="(value) => $emit('update:expansion', value)" label="Parametric filters"
            header-class="title-bar-lv1">
            <q-card-section class="q-pb-none">
                <div class="info-box">
                    Parametric filters are audio processing filters running on the RP2040 chip in the DAC. This chip has a
                    limited amount of processing power and if you try to enable too many filters you might find it starts
                    dropping audio samples.
                </div>
            </q-card-section>
            <q-card-section class="q-py-sm">
                <q-list class="col-12">
                    <q-item style="padding-left:0px; padding-right:0px" v-for="filter in filters">
                        <FilterVue v-model:filter_type="filter.filter_type" v-model:f0="filter.f0"
                            v-model:db_gain="filter.db_gain" v-model:q="filter.q" 
                            v-model:a0="filter.a0" v-model:a1="filter.a1" v-model:a2="filter.a2"
                            v-model:b0="filter.b0" v-model:b1="filter.b1" v-model:b2="filter.b2"
                            v-model:enabled="filter.enabled"
                            @delete:filter="deleteFilter(filter)" ref="filter" />
                    </q-item>
                </q-list>
                <div class="row">
                    <q-btn fab icon="add" label="New Filter" color="primary" @click="addFilter()"
                        :disable="this.filters.length >= 32" />
                </div>
            </q-card-section>
        </q-expansion-item>
    </q-card>
</template>
