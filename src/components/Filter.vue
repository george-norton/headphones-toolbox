<script setup>
  import { FilterTypes } from './FilterTypes.js'
</script>

<template>
  <div class="full-width">
    <q-toolbar class="title-bar-lv2">
      <q-select dense stretch flat :model-value="filter_type" map-options option-value="value" option-label="label"
        @update:model-value="(value) => $emit('update:filter_type', value.value)" :options="filter_types" class="col-2">
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
      <q-list dense bordered class="col-grow q-py-sm" v-if="filter_type !== FilterTypes.CUSTOMIIR">
        <q-item>
          <q-item-section>
            <div class = "row items-center">
              <div class="col-3 justify-start items-center q-gutter-sm">
                <q-chip icon="graphic_eq" class="control-label" color=secondary text-color=white>Frequency</q-chip>
                <!--q-item-label caption lines="2">The centre frequency. this is where the signal starts getting attenuated.</q-item-label-->
              </div>
              <div class="col">
                <!-- Use a logarithmic scale here as this is how the graph is plotted. It makes picking low frequencies easier. -->
                <q-slider :model-value="Math.log(f0) / Math.log(20000)"
                @update:model-value="(value) => $emit('update:f0', Math.pow(20000, value))" label
                :label-value="Math.round(f0 * 100) / 100 + 'hz'" :min=0 :max=1 :step=0.001>
              </q-slider>
            </div>
          </div>
          </q-item-section>
          <q-item-section side>
            <q-input type="number" dense hide-bottom-space shadow-text="hz" style="width:5em"
              :model-value="Math.round(f0)" @update:model-value="(value) => $emit('update:f0', Number(value))" :min=1
              :max=20000 :rules="[val => (val >= 1 && val <= 20000) || 'Frequency out of range']" :debounce=1000 />
          </q-item-section>
        </q-item>

        <q-item v-if="[FilterTypes.LOWSHELF, FilterTypes.HIGHSHELF, FilterTypes.PEAKING].includes(filter_type)">

          <q-item-section>
            <div class = "row">
              <div class="col-3 justify-start items-center q-gutter-sm">
                <q-chip icon="volume_up" class="control-label" color=secondary text-color=white>Gain</q-chip>
                <!--q-item-label caption lines="2">The gain at the centre frequency, in dB. Positive for boost, negative for
                cut.</q-item-label-->
              </div>
              <div class="col">
                <q-slider :model-value="db_gain" @update:model-value="(value) => $emit('update:db_gain', value)" :min=-25
                  :max=25 :step=0.01 label :label-value="db_gain + 'db'" />
                </div>
              </div>
          </q-item-section>

          <q-item-section side>
            <q-input type="number" dense hide-bottom-space shadow-text="db" style="width:5em" :model-value="db_gain"
              @update:model-value="(value) => $emit('update:db_gain', Number(value))" :min=-25 :max=25
              :rules="[val => (val >= -25 && val <= 25) || 'Gain out of range']" :debounce=1000 />
          </q-item-section>
        </q-item>
        <q-item>

          <q-item-section>
            <div class = "row">
              <div class="col-3 justify-start items-center q-gutter-sm">
                <q-chip icon="auto_graph" class="control-label" color=secondary text-color=white>Quality</q-chip>
                <!--q-item-label caption lines="2">The quality factor. It defines how aggressive the band pass attenuates from the centre frequency. When Q=sqrt(2) it is 1 octave wide</q-item-label-->
              </div>
              <div class="col">
                <q-slider :model-value="q" @update:model-value="(value) => $emit('update:q', value)" :min=0 :max=33 :step=0.01
                  :inner-min=0.1 label />
                </div>
            </div>
          </q-item-section>
          <q-item-section side>
            <q-input type="number" dense hide-bottom-space style="width:5em" :model-value="q"
              @update:model-value="(value) => $emit('update:q', Number(value))" :min=0 :max=33
              :rules="[val => (val >= 0 && val <= 33) || 'Quality out of range']" :debounce=1000 />
          </q-item-section>
        </q-item>
      </q-list>

      <q-list dense bordered class="col" v-else>
        <div class="info-box  q-ma-sm">
          <q-icon size="sm" name="warning" color="red" />
          Warning! Do not mess with these coefficients while you are listening to your headphones. It will get loud!
        </div>
        <div class="row fit justify-start q-pa-sm">
          <div class="col-shrink q-mx-sm">
            <q-chip icon="arrow_backward" class="control-label" color=secondary text-color=white>Feed Backward
              Coefficients</q-chip>
            <div class="row justify-start q-gutter-md">
              <q-input label="a0" type="number" dense hide-bottom-space style="width:8em" :step="1.0 / iirDp"
                :debounce=100 :model-value="a0" @update:model-value="(value) => $emit('update:a0', Number(value))"
                input-class="truncate-text" :rules="[val => (val != 0) || 'The a0 coefficient must not be zero',
                val => (val >= -32 && val < 32) || 'Coefficient cannot be represented by the fixed point data type.']"
                :min=-32 :max=32 />
              <q-input label="a1" type="number" dense hide-bottom-space style="width:8em" :step="1.0 / iirDp"
                :debounce=100 :model-value="a1" @update:model-value="(value) => $emit('update:a1', Number(value))"
                input-class="truncate-text" :min=-32 :max=32
                :rules="[val => (val >= -32 && val < 32) || 'Coefficient cannot be represented by the fixed point data type.']" />
              <q-input label="a2" type="number" dense hide-bottom-space style="width:8em" :step="1.0 / iirDp"
                :debounce=100 :model-value="a2" @update:model-value="(value) => $emit('update:a2', Number(value))"
                input-class="truncate-text" :min=-32 :max=32
                :rules="[val => (val >= -32 && val < 32) || 'Coefficient cannot be represented by the fixed point data type.']" />
            </div>
          </div>

          <div class="col-shrink q-mx-sm">
            <q-chip icon="arrow_forward" class="control-label" color=secondary text-color=white>Feed Forward
              Coefficients</q-chip>
            <div class="row justify-start q-gutter-md">
              <q-input label="b0" type="number" dense hide-bottom-space style="width:8em" :step="1.0 / iirDp"
                :debounce=100 :model-value="b0" @update:model-value="(value) => $emit('update:b0', Number(value))"
                input-class="truncate-text" :min=-32 :max=32
                :rules="[val => (val >= -32 && val < 32) || 'Coefficient cannot be represented by the fixed point data type.']" />
              <q-input label="b1" type="number" dense hide-bottom-space style="width:8em" :step="1.0 / iirDp"
                :debounce=100 :model-value="b1" @update:model-value="(value) => $emit('update:b1', Number(value))"
                input-class="truncate-text" :min=-32 :max=32
                :rules="[val => (val >= -32 && val < 32) || 'Coefficient cannot be represented by the fixed point data type.']" />
              <q-input label="b2" type="number" dense hide-bottom-space style="width:8em" :step="1.0 / iirDp"
                :debounce=100 :model-value="b2" @update:model-value="(value) => $emit('update:b2', Number(value))"
                input-class="truncate-text" :min=-32 :max=32
                :rules="[val => (val >= -32 && val < 32) || 'Coefficient cannot be represented by the fixed point data type.']" />
            </div>
          </div>
        </div>
      </q-list>
    </div>
  </div>
</template>
 
<script>
import { ref } from 'vue'
import { getCssVar } from 'quasar'
import { getFilterCoefficients } from '@/components/FilterCoefficients.js'

export default {
  watch: {
    filter_type(new_type, old_type) {
      if (new_type == FilterTypes.CUSTOMIIR) {
        var c = getFilterCoefficients(old_type, this.f0, this.db_gain, this.q)
        if (c) {
          this.$emit('update:a0', Math.round(c.feedback[0] * this.iirDp) / this.iirDp)
          this.$emit('update:a1', Math.round(c.feedback[1] * this.iirDp) / this.iirDp)
          this.$emit('update:a2', Math.round(c.feedback[2] * this.iirDp) / this.iirDp)
          this.$emit('update:b0', Math.round(c.feedforward[0] * this.iirDp) / this.iirDp)
          this.$emit('update:b1', Math.round(c.feedforward[1] * this.iirDp) / this.iirDp)
          this.$emit('update:b2', Math.round(c.feedforward[2] * this.iirDp) / this.iirDp)
        }
      }
    }
  },
  data() {
    return {
      iirDp: 1000000000,
      filter_types: [{ value: FilterTypes.LOWPASS, label: 'Low Pass' },
      { value: FilterTypes.HIGHPASS, label: 'High Pass' },
      { value: FilterTypes.BANDPASSSKIRT, label: 'Bandpass Skirt' },
      { value: FilterTypes.BANDPASSPEAK, label: 'Bandpass Peak' },
      { value: FilterTypes.NOTCH, label: "Notch" },
      { value: FilterTypes.ALLPASS, label: "All Pass" },
      { value: FilterTypes.PEAKING, label: "Peaking" },
      { value: FilterTypes.LOWSHELF, label: "Low Shelf" },
      { value: FilterTypes.HIGHSHELF, label: "High Shelf" },
      { value: FilterTypes.CUSTOMIIR, label: "Custom IIR Filter" }],
      warning: ref(true)
    }
  },
  props: {
    filter_type: ref(String),
    f0: ref(Number),
    db_gain: ref(Number),
    q: ref(Number),
    a0: ref(Number),
    a1: ref(Number),
    a2: ref(Number),
    b0: ref(Number),
    b1: ref(Number),
    b2: ref(Number),
    enabled: ref(Boolean),
    expansion: ref(Boolean)
  },
  emits: ['update:filter_type', 'update:f0', 'update:db_gain', 'update:q', 'update:a0', 'update:a1', 'update:a2', 'update:b0', 'update:b1', 'update:b2', 'update:enabled', 'delete:filter']
}
</script>