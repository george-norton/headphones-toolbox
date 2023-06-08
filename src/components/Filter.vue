<template>
  <div class="full-width">
    <q-toolbar class="title-bar-lv2">
      <q-select dense stretch flat :model-value="filter_type"
        @update:model-value="(value) => $emit('update:filter_type', value)" :options="filter_types" class="col-2">
        <template v-slot:prepend>
          <q-icon name="equalizer" />
        </template>
      </q-select>
      <q-toolbar-title>
      </q-toolbar-title>
      <q-toggle dense :model-value="enabled" @update:model-value="(value) => $emit('update:enabled', value)"
        checked-icon="music_note" unchecked-icon="music_off">
        <q-tooltip>
          Enable/Disable this filter
        </q-tooltip>
      </q-toggle>
      <q-btn flat round dense icon="delete" @click="$emit('delete:filter')" class="title-bar-lv2">
        <q-tooltip>
          Delete this filter
        </q-tooltip>
      </q-btn>
    </q-toolbar>
    <div class="full-width row">
      <q-list dense bordered class="col-grow q-py-sm">
        <q-item>
          <q-item-section>
            <div class="col">
              <q-chip class="control-label" color=secondary text-color=white>Frequency</q-chip>
            </div>
            <!-- Use a logarithmic scale here as this is how the graph is plotted. It makes picking low frequencies easier. -->
            <q-slider :model-value="Math.log(f0) / Math.log(20000)"
              @update:model-value="(value) => $emit('update:f0', Math.pow(20000, value))" label
              :label-value="Math.round(f0 * 100) / 100 + 'hz'" :min=0 :max=1 :step=0.001>
            </q-slider>
          </q-item-section>
          <q-item-section side>
            <q-input type="number" dense hide-bottom-space shadow-text="hz" style="width:5em"
              :model-value="Math.round(f0)" @update:model-value="(value) => $emit('update:f0', Number(value))" :min=1
              :max=20000 :rules="[val => (val >= 1 && val <= 20000) || 'Frequency out of range']" :debounce=1000 />
          </q-item-section>
        </q-item>
        <q-item v-if="['lowshelf', 'highshelf', 'peaking'].includes(filter_type)">
          <q-item-section>
            <div class="col">
              <q-chip class="control-label" color=secondary text-color=white>Gain (dB)</q-chip>
            </div>
            <q-slider :model-value="db_gain" @update:model-value="(value) => $emit('update:db_gain', value)" :min=-20
              :max=20 :step=0.01 label :label-value="db_gain + 'db'" />
          </q-item-section>
          <q-item-section side>
            <q-input type="number" dense hide-bottom-space shadow-text="db" style="width:5em" :model-value="db_gain"
              @update:model-value="(value) => $emit('update:db_gain', Number(value))" :min=-20 :max=20
              :rules="[val => (val >= -20 && val <= 20) || 'Gain out of range']" :debounce=1000 />
          </q-item-section>
        </q-item>
        <q-item>

          <q-item-section>
            <div class="col">
              <q-chip class="control-label" color=secondary text-color=white>Quality</q-chip>
            </div>
            <q-slider :model-value="q" @update:model-value="(value) => $emit('update:q', value)" :min=0 :max=33 :step=0.01
              label />
          </q-item-section>
          <q-item-section side>
            <q-input type="number" dense hide-bottom-space style="width:5em" :model-value="q"
              @update:model-value="(value) => $emit('update:q', Number(value))" :min=0 :max=33
              :rules="[val => (val >= 0 && val <= 33) || 'Quality out of range']" :debounce=1000 />
          </q-item-section>
        </q-item>
      </q-list>
    </div>
  </div>
</template>
 
<script>
import { ref, toRefs } from 'vue'
import { getCssVar } from 'quasar'

export default {
  data() {
    return {
      // bandpass_skirt is not supported by the web audio api, so we cant generate a graph for it.
      filter_types: ['lowpass', 'highpass', /*'bandpass_skirt', 'bandpass_peak'*/ 'bandpass', 'notch', 'allpass', 'peaking', 'lowshelf', 'highshelf'],
    }
  },
  props: {
    filter_type: ref(String),
    f0: ref(Number),
    db_gain: ref(Number),
    q: ref(Number),
    enabled: ref(Boolean)
  },
  emits: ['update:filter_type', 'update:f0', 'update:db_gain', 'update:q', 'update:enabled', 'delete:filter']
}
</script>