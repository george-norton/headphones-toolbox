<template>
  <div class="q-pa-none">
    <q-btn dense flat icon="info" :disable="disable">
      <q-tooltip>
        Information about this device.
      </q-tooltip>
      <q-menu>
        <q-table flat bordered dense :rows-per-page-options="[0]" row_key="key" :rows="rows" :columns="columns"
          separator="vertical" hide-header hide-bottom :loading="loading">
          <template v-slot:top>
            <div class="bold-heading">Device Info</div>
          </template>
        </q-table>
      </q-menu>
    </q-btn>
  </div>
</template>
  
<script>
import { ref, reactive } from 'vue'

export default {
  setup() {
    return {
    }
  },
  props: {
    disable: ref(Boolean),
    versions: ref({ "git_hash": "" })
  },
  watch: {
    versions: function (newVal, oldVal) { // watch it
      this.rows[0].value = newVal.serial_number;
      this.rows[1].value = newVal.git_hash;
      this.rows[2].value = newVal.pico_sdk_version;
      this.rows[3].value = newVal.minimum_supported_version;
      this.rows[4].value = newVal.current_version;
      this.rows[5].value = newVal.client_api_version;
    }
  },
  data() {
    return {
      loading: false,
      rows: [
        { key: "Serial Number:", value: this.versions ? this.versions.serial_number : "Unknown" },
        { key: "Firmware Git Hash:", value: this.versions ? this.versions.git_hash : "Unknown" },
        { key: "Pico SDK Version:", value: this.versions ? this.versions.pico_sdk_version : "Unknown" },
        { key: "Min API version:", value: this.versions ? this.versions.minimum_supported_version : "Unknown" },
        { key: "Max API version:", value: this.versions ? this.versions.current_version : "Unknown" },
        { key: "Client API version:", value: this.versions ? this.versions.client_api_version : "Unknown" },
      ],
      columns: [
        { name: "key", field: "key", align: 'right', style: "font-weight: bold" },
        { name: "value", field: "value", align: 'left', style: "font-weight: normal" }
      ]
    }
  }
}
</script>