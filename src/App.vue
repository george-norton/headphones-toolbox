<script setup>
import GraphVue from './components/Graph.vue'
import FilterCardVue from './components/FilterCard.vue'
import { appWindow } from '@tauri-apps/api/window'

</script>

<script>
import { ref, reactive } from 'vue'
import { exportFile, getCssVar } from 'quasar'
import { invoke } from '@tauri-apps/api'
var idSequence = 0
var deviceNames = { "none": "No device detected" }
var deviceListKey = ref(0)

export default {
  setup() {

  },
  mounted() {
    this.loadState()
    this.pollDevices()
    window.setInterval(this.pollDevices, 5000)
  },
  unmounted() {
    this.saveState()
  },
  watch: {
    tab() {
      this.saveState()
    },
    tabs: {
      handler() {
        this.saveState()
      },
      deep: true
    },
    file() {
      const reader = new FileReader();
      const data = reader.readAsText(this.file);
      reader.onload = () => {
        try {
          const configData = JSON.parse(reader.result);
          var nextId = this.tabs.length
          configData.id = nextId
          if (configData.name && configData.configuration) {
            this.tabs.push(configData)
            this.tab = nextId
          }
          else {
            throw new SyntaxError("Missing JSON elements");
          }
        } catch (err) {
          this.$q.notify({ type: 'negative', message: "Failed to load config" })
          console.log(err)
        }
      };
    }
  },
  data() {
    return {
      file: ref(undefined),
      tab: ref(0),
      tabs: reactive([]),
      devices: reactive([]),
      deviceOptions: reactive([]),
      device: ref(undefined),
      connected: ref(undefined)
    }
  },
  components: {
    FilterCardVue
  },
  methods: {
    pageHeight(offset) {
      const height = offset ? `calc(100vh - ${offset}px)` : '100vh'
      return { height: height }
    },
    addConfiguration() {
      var nextId = this.tabs.length
      this.tabs.push({ id: nextId, name: "Unnamed configuration", configuration: [] })
      this.tab = nextId
    },
    deleteConfiguration() {
      for (var i = 0; i < this.tabs.length; i++) {
        if (this.tabs[i].id == this.tab) {
          this.tabs.splice(i, 1)
          if (i > 0) {
            this.tab = this.tabs[i - 1].id
          }
          else if (i < this.tabs.length) {
            this.tab = this.tabs[i].id
          }
        }
        this.tabs[i].id = i
      }
    },
    async saveState() {
      var config = {
        "currentConfiguration": this.tab,
        "configurations": this.tabs,
        "deviceNames": deviceNames
      }
      localStorage.setItem("state", JSON.stringify(config))
    },
    loadState() {
      var config = JSON.parse(localStorage.getItem("state"))
      if (config) {
        for (var c in config.configurations) {
          if (config.configurations[c].id == config.currentConfiguration) {
            this.tab = c
          }
          config.configurations[c].id = c
        }
        this.tabs = reactive(config.configurations)
        deviceNames = config.deviceNames
      }
    },
    exportConfiguration() {
      const config = JSON.stringify(this.tabs[this.tab], null, 4)
      exportFile(this.tabs[this.tab].name + ".json", config)
    },
    importConfiguration() {
      this.$refs.importFile.pickFiles()
    },
    updateDeviceName(name) {
      deviceNames[this.device] = name
      deviceListKey.value += 1
      this.saveState()
    },
    pollDevices() {
      invoke('poll_devices').then((message) => {
        var devices = JSON.parse(message)
        for (var d in devices) {
          if (!(d in deviceNames)) {
            if (devices.length == 1 && !("Ploopy Headphones" in deviceNames)) {
              // Most people will only have one device, so use a friendly name
              deviceNames[devices[d]] = "Ploopy Headphones"
            }
            else {
              deviceNames[devices[d]] = "Headphones [" + devices[d] + "]"
            }
          }
        }
        Object.assign(this.devices, devices)

        if ((this.device == undefined || this.device == "none") && this.devices.length > 0) {
          this.device = this.devices[0];
        }
        else {
          if (this.device == undefined && this.devices.length == 0) {
            this.$q.notify({ type: 'negative', message: "No devices detected" })
            this.device = "none"
            this.connected = false
          }
          else if (this.device != "none") {
            if (this.connected && (devices.indexOf(this.device) == -1)) {
              this.$q.notify({ type: 'negative', message: "Device disconnected" })
              this.connected = false
            }
            else if (!this.connected) {
              if (devices.indexOf(this.device) != -1) {
                this.$q.notify({ type: 'positive', message: "Device connected" })
              }
              this.connected = true
              invoke('open', {serialNumber: this.device}).then((message) => {
                console.log("Open returned " + message)
              })
            }
          }
        }
      })
    }
  }
}
</script>
<template>
  <q-layout view="hHh lpR fFf">
    <q-header elevated class="bg-primary text-white">

      <q-bar data-tauri-drag-region>
        <q-icon style="pointer-events: none;" name="img:src/assets/ploopy.png" />
        <div style="pointer-events: none;">Ploopy Headphones Toolbox</div>
        <q-space />
        <q-btn dense flat icon="minimize" @click="appWindow.minimize()" />
        <q-btn dense flat icon="crop_square" @click="appWindow.maximize()" />
        <q-btn dense flat icon="close" @click="appWindow.close()" />
      </q-bar>

      <q-toolbar class="bg-primary text-white justify-start">
        <q-select filled v-model="device" :key="deviceListKey" :options="devices" option-value="value"
          :option-label="item => deviceNames[item]" map-options dark options-dark="false" bg-color="primary"
          ref="deviceSelect">
          <template v-slot:prepend>
            <q-icon name="headphones" />
          </template>
        </q-select>
        <q-btn flat dense icon="edit" :disable="!connected">
          <q-tooltip>
            Rename this device.
          </q-tooltip>
          <q-popup-edit v-slot="devicePopup" anchor="center middle" self="top middle" :cover="false">
            <q-input v-model="deviceNames[device]" dense autofocus @keyup.enter="devicePopup.set"
              @focus="(input) => input.target.select()" @update:model-value="$value => updateDeviceName($value)" />
          </q-popup-edit>
        </q-btn>
        <q-btn flat dense icon="restart_alt" :disable="!connected" @click="invoke('reboot_bootloader')">
          <q-tooltip>
            Reboot this device into the bootloader so you can install new firmware.
          </q-tooltip>
        </q-btn>
        <q-btn flat dense icon="delete" :disable="!connected">
          <q-tooltip>
            Reset the device to its factory default settings.
          </q-tooltip>
        </q-btn>
        <q-space />
        <q-btn flat dense icon="save_alt" :disable="!connected">
          <q-tooltip>
            Persist the current configuration to flash memory on the DAC.
          </q-tooltip>
        </q-btn>
      </q-toolbar>

    </q-header>

    <q-page-container>
      <q-page :style-fn="pageHeight" class="scroll overflow-auto">

        <q-tabs v-model="tab" dense align="left" :breakpoint="0" class="bg-grey-1 text-black" ref="tabs">
          <q-tab v-for="t in tabs" :name="t.id" :label="t.name" />
          <q-btn flat dense icon="add" text-color="grey-9" @click="$event => addConfiguration()">
            <q-tooltip>
              Add a new configuration.
            </q-tooltip>
          </q-btn>

          <q-separator vertical inset />

          <q-btn flat dense icon="edit" text-color="grey-9">
            <q-tooltip>
              Rename this configuration.
            </q-tooltip>
            <q-popup-edit auto-save v-slot="tabPopup" anchor="center middle" self="top middle" :cover="false">
              <q-input v-model="tabs[tab].name" dense autofocus @keyup.enter="tabPopup.set"
                @focus="(input) => input.target.select()" />
            </q-popup-edit>
          </q-btn>
          <q-btn flat dense icon="delete" text-color="grey-9" @click="$event => deleteConfiguration()">
            <q-tooltip>
              Delete this configuration.
            </q-tooltip>
          </q-btn>

          <q-space />

          <q-btn flat dense icon="file_download" text-color="grey-9" @click="importConfiguration()">
            <q-tooltip>
              Import a configuration from a JSON file.
            </q-tooltip>
            <q-file ref="importFile" class="hidden" accept=".json" clearable filled v-model="file" />
          </q-btn>
          <q-btn flat dense icon="file_upload" text-color="grey-9" @click="exportConfiguration()">
            <q-tooltip>
              Export this configuration to a JSON file.
            </q-tooltip>
          </q-btn>
          <q-btn flat dense icon="usb" text-color="grey-9" @click="">
            <q-tooltip>
              Import configuration from the connected device.
            </q-tooltip>
          </q-btn>
          <!--q-btn flat dense icon="more_vert" text-color="grey-9">
                              <q-menu>
                                <q-list style="min-width: 100px">
                                  <q-item clickable v-close-popup>
                                    <q-item-section>Export to JSON</q-item-section>
                                  </q-item>
                                  <q-item clickable v-close-popup>
                                    <q-item-section>Import from JSON</q-item-section>
                                  </q-item>
                                  <q-item clickable v-close-popup>
                                    <q-item-section>Import from device</q-item-section>
                                  </q-item>
                                </q-list>
                              </q-menu>
                            </q-btn-->

        </q-tabs>
        <q-tab-panels v-model="tab" animated class="bg-grey-1">
          <q-tab-panel v-for="t in tabs" :name="t.id" class="column q-gutter-md q-ma-none bg-grey-1">
            <FilterCardVue v-model:filters="t.configuration" ref="filterCard" />
          </q-tab-panel>
        </q-tab-panels>

      </q-page>
    </q-page-container>

    <q-footer elevated class="bg-grey-8 text-white">

      <div class="block full-width bg-white">
        <GraphVue ref="graph" v-model:filters="tabs[tab]" />
      </div>

    </q-footer>

  </q-layout>
</template>

 
