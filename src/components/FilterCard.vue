
<script>
import FilterVue from './Filter.vue'

export default {
    props: {
        filters: {
            type: Array,
            default: []
        }
    },
    methods: {
        addFilter() {
            this.filters.push({ filter_type: "lowpass", q: 0, f0: 0, db_gain: 0, enabled: true })
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
            <div class="text-h6">Parametric filters</div>
        </q-card-section>
        <q-card-section class="q-py-sm">
            <q-list class="col-12">
                <q-item style="padding-left:0px; padding-right:0px" v-for="filter in filters">
                    <FilterVue v-model:filter_type="filter.filter_type" v-model:f0="filter.f0"
                        v-model:db_gain="filter.db_gain" v-model:q="filter.q" v-model:enabled="filter.enabled"
                        @delete:filter="deleteFilter(filter)" ref="filter" />
                </q-item>
            </q-list>
            <div class="row">
                <q-btn fab icon="add" label="New Filter" color="primary" @click="addFilter()"
                    :disable="this.filters.length >= 8" />
            </div>
        </q-card-section>
    </q-card>
</template>
